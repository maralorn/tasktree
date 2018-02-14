extern crate gtk;
extern crate glib;
extern crate gdk;
extern crate task_hookrs;
extern crate uuid;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

#[macro_use]
extern crate lazy_static;
extern crate regex;

mod util;
mod task;
use util::{run, Result};
use gtk::prelude::*;
use task_hookrs::status::TaskStatus;
use uuid::Uuid;

struct TaskTreeState {
    treestore: gtk::TreeStore,
    filterbuffer: gtk::EntryBuffer,
    tasks: task::TaskCache,
    positions: std::collections::HashMap<Uuid, gtk::TreeIter>,
}

impl TaskTreeState {
    fn show_task(&mut self, uuid: &Uuid) -> Result<gtk::TreeIter> {
        if let Some(position) = self.positions.get(uuid) {
            return Ok(position.clone());
        }

        let partof_uuid = self.tasks.get_task(uuid)?.partof;
        let partof_iter = match partof_uuid.as_ref() {
            Some(uuid) => Some(self.show_task(uuid)?),
            _ => None,
        };
        let task = self.tasks.get_task(uuid)?;

        let iter = self.treestore.insert_with_values(
            partof_iter.as_ref(),
            None,
            &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
            &[
                &task.uuid.to_string(),
                &task.description,
                &match task.status {
                    TaskStatus::Pending => "PENDING",
                    TaskStatus::Deleted => "DELETED",
                    TaskStatus::Completed => "COMPLETED",
                    TaskStatus::Waiting => "WAITING",
                    TaskStatus::Recurring => "RECURRING",
                },
                &(task.status == TaskStatus::Completed),
                &(task.status == TaskStatus::Deleted),
                &match task.tags {
                    Some(ref tags) => tags.join(", "),
                    None => "".to_string(),
                },
                &task.project,
                &match task.due {
                    Some(ref date) => date.format("%F %R").to_string(),
                    None => "".to_string(),
                },
                &match task.wait {
                    Some(ref date) => date.format("%F %R").to_string(),
                    None => "".to_string(),
                },
                &task.entry.format("%F %R").to_string(),
                &match task.modified {
                    Some(ref date) => date.format("%F %R").to_string(),
                    None => "".to_string(),
                },
            ],
        );
        self.positions.insert(*uuid, iter.clone());
        Ok(iter)
    }
    pub fn refresh(&mut self) {
        util::run(|| {
            self.positions.clear();
            self.treestore.clear();
            self.tasks.refresh()?;
            for task in task::get_tasks(&self.filterbuffer.get_text())? {
                self.show_task(&task)?;
            }
            Ok(())

        });
    }
    /// This function will update the app cache and the task cache for a changed task.
    /// If you pass a `TreeIter` it will just assume that the display is already correct and save
    /// it to the cache. If you don’t it will refresh the line.
    pub fn update(&mut self, uuid: &uuid::Uuid, new_iter: Option<&gtk::TreeIter>) -> Result<()> {
        self.tasks.update(uuid)?;
        if let Some(new_iter) = new_iter {
            self.positions.insert(*uuid, new_iter.clone());
        } else {
            let iter = self.positions.remove(uuid).ok_or(
                "trying to update task with unknown position",
            )?;
            self.treestore.remove(&iter);
            self.show_task(uuid)?;
        }
        Ok(())
    }
}

fn main() {

    if gtk::init().is_err() {
        return println!("Failed to initialize GTK.");
    }
    gdk::set_program_class("Tasktree");
    // First we get the file content.
    let glade_src = include_str!("tasklist.glade");
    // Then we call the Builder call.
    let builder = gtk::Builder::new_from_string(glade_src);


    let window: gtk::Window = builder.get_object("mainwindow").unwrap();
    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        gtk::Inhibit(false)
    });


    let app = std::rc::Rc::new(std::cell::RefCell::new(TaskTreeState {
        treestore: builder.get_object("tasktree").unwrap(),
        filterbuffer: builder.get_object("filterbuffer").unwrap(),
        tasks: task::TaskCache::new(),
        positions: std::collections::HashMap::new(),
    }));
    let filter_field: gtk::SearchEntry = builder.get_object("filterfield").unwrap();
    let app_pointer = app.clone();
    filter_field.connect_key_press_event(move |_, eventkey| {
        util::run(|| {
            if eventkey.get_keyval() == gdk::enums::key::Return {
                app_pointer.try_borrow_mut()?.refresh()
            };
            Ok(())
        });
        gtk::Inhibit(false)
    });
    let app_pointer = app.clone();
    let done_cell: gtk::CellRendererToggle = builder.get_object("done-cell").unwrap();
    done_cell.connect_toggled(move |_, treepath| {
        util::run(|| {
            let mut app = app_pointer.try_borrow_mut()?;
            let iter = app.treestore.get_iter(&treepath).ok_or(
                "Treepath didn’t give us an Iter",
            )?;
            let uuid_val = app.treestore.get_value(&iter, 0);
            let uuid_str = uuid_val.get().ok_or(
                "Didn’t get correct uuid_str from treestore",
            )?;
            let uuid = Uuid::parse_str(uuid_str)?;
            match app.tasks.get_task(&uuid)?.status {
                TaskStatus::Completed => task::pending(&uuid)?,
                TaskStatus::Pending => task::done(&uuid)?,
                TaskStatus::Recurring => task::done(&uuid)?,
                TaskStatus::Waiting => task::done(&uuid)?,
                TaskStatus::Deleted => (),
            };
            app.update(&uuid, None)
        });
    });
    let app_pointer = app.clone();
    let deleted_cell: gtk::CellRendererToggle = builder.get_object("deleted-cell").unwrap();
    deleted_cell.connect_toggled(move |_, treepath| {
        util::run(|| {
            let mut app = app_pointer.try_borrow_mut()?;
            let iter = app.treestore.get_iter(&treepath).ok_or(
                "Treepath didn’t give us an Iter",
            )?;
            let uuid_val = app.treestore.get_value(&iter, 0);
            let uuid_str = uuid_val.get().ok_or(
                "Didn’t get correct uuid_str from treestore",
            )?;
            let uuid = uuid::Uuid::parse_str(uuid_str)?;
            match app.tasks.get_task(&uuid)?.status {
                TaskStatus::Completed => task::delete(&uuid)?,
                TaskStatus::Pending => task::delete(&uuid)?,
                TaskStatus::Recurring => task::delete(&uuid)?,
                TaskStatus::Waiting => task::delete(&uuid)?,
                TaskStatus::Deleted => task::pending(&uuid)?,
            };
            app.update(&uuid, None)
        });
    });
    let app_pointer = app.clone();
    let new_child_cell: gtk::CellRendererText = builder.get_object("new-child-cell").unwrap();
    new_child_cell.connect_edited(move |_, treepath, description| if description.len() > 0 {
        run(|| {
            let mut app = app_pointer.try_borrow_mut()?;
            let iter = app.treestore.get_iter(&treepath).ok_or(
                "Treepath didn’t give us an Iter",
            )?;
            let uuid_val = app.treestore.get_value(&iter, 0);
            let uuid_str = uuid_val.get().ok_or(
                "Didn’t get correct uuid_str from treestore",
            )?;
            let uuid = uuid::Uuid::parse_str(uuid_str)?;
            let new_task = app.tasks.create(description, Some(&uuid))?.uuid;
            app.show_task(&new_task).map(|_| ())
        });
    });
    let app_pointer = app.clone();
    let description_cell: gtk::CellRendererText = builder.get_object("description-cell").unwrap();
    description_cell.connect_edited(move |_, treepath, description| if description.len() > 0 {
        run(|| {
            let app = app_pointer.try_borrow()?;
            let iter = app.treestore.get_iter(&treepath).ok_or(
                "Treepath didn’t give us an Iter",
            )?;
            let uuid_val = app.treestore.get_value(&iter, 0);
            let uuid_str = uuid_val.get().ok_or(
                "Didn’t get correct uuid_str from treestore",
            )?;
            let uuid = uuid::Uuid::parse_str(uuid_str)?;
            let app_pointer = app_pointer.clone();
            gtk::idle_add(move || {
                run(|| app_pointer.try_borrow_mut()?.update(&uuid, None));
                glib::Continue(false)
            });
            task::set_description(&uuid, description)
        });
    });
    util::run(|| {
        let mut borrowed_app = app.try_borrow_mut()?;
        let app_pointer = app.clone();
        borrowed_app.treestore.connect_row_changed(
            move |_, _, iter| {
                run(|| {
                    let mut app = match app_pointer.try_borrow_mut() {
                        Ok(app) => app,
                        Err(_) => return Ok(()), // This propably means, that we are inside show_task
                    };
                    let uuid_val = app.treestore.get_value(&iter, 0);
                    let uuid_str = uuid_val.get().ok_or(
                        "Didn’t get correct uuid_str from treestore",
                    )?;
                    let uuid = uuid::Uuid::parse_str(uuid_str)?;
                    let parent = match app.treestore.iter_parent(&iter) {
                        Some(iter_parent) => {
                            let parent_val = app.treestore.get_value(&iter_parent, 0);
                            let parent_str = parent_val.get().ok_or(
                                "Didn’t get correct parent_uuid_str from treestore",
                            )?;
                            Some(uuid::Uuid::parse_str(parent_str)?)
                        }
                        None => None,
                    };
                    if app.tasks.get_task(&uuid)?.partof != parent {
                        task::partof(&uuid, parent.as_ref())?;
                        app.update(&uuid, Some(iter))
                    } else {
                        Ok(())
                    }
                });
            },
        );
        Ok(borrowed_app.refresh())
    });
    window.show_all();
    gtk::main();
}
