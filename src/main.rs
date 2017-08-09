extern crate gtk;
extern crate task_hookrs;
extern crate uuid;

type Result<T> = std::result::Result<T, Box<std::error::Error>>;

struct TaskTreeState {
    treestore: gtk::TreeStore,
    filterbuffer: gtk::EntryBuffer,
    tasks: std::collections::HashMap<uuid::Uuid, (task_hookrs::task::Task, Option<gtk::TreeIter>)>,
}

impl TaskTreeState {
    fn show_task(&mut self, uuid: uuid::Uuid) -> Result<gtk::TreeIter> {
        let parent_uuid = {
            match self.tasks.get(&uuid).ok_or("uuid not found in HashMap")? {
                &(_, Some(ref position)) => return Ok(position.clone()),
                &(ref task, _) => task.parent().cloned(),
            }
        };
        let parent_iter = match parent_uuid {
            Some(uuid) => Some(self.show_task(uuid)?),
            _ => None,
        };
        let mut taskstate = self.tasks.get_mut(&uuid).ok_or("uuid not found in HashMap")?;
        taskstate.1 = Some(self.treestore.insert_with_values(
            parent_iter.as_ref(),
            None,
            &[0],
            &[taskstate.0.description()],
        ));
        Ok(taskstate.1.clone().unwrap())
    }
    pub fn refresh(&mut self) {
        let mut refresh = || -> Result<()> {
            let mut child = std::process::Command::new("task")
                .arg("export")
                .stdout(std::process::Stdio::piped())
                .spawn()?;
            let stdout = child.stdout.take().ok_or("Didn’t capture stdout")?;
            let tasks = task_hookrs::import::import(stdout)?;
            child.wait()?;
            for task in tasks {
                self.tasks.insert(*task.uuid(), (task, None));
            }
            let stdout = &std::process::Command::new("task")
                .arg("_uuid")
                .arg(self.filterbuffer.get_text() + "+PENDING")
                .output()?
                .stdout;
            for uuid_str in std::str::from_utf8(stdout)?.split_whitespace() {
                self.show_task(uuid::Uuid::parse_str(uuid_str)?)?;
            }
            Ok(())

        };

        match refresh() {
            Err(err) => println!("Error: {}", err.description()),
            _ => (),
        };

    }
}

fn main() {

    use gtk::WidgetExt;

    if gtk::init().is_err() {
        return println!("Failed to initialize GTK.");
    }
    // First we get the file content.
    let glade_src = include_str!("tasklist.glade");
    // Then we call the Builder call.
    let builder = gtk::Builder::new_from_string(glade_src);


    let window: gtk::Window = builder.get_object("mainwindow").unwrap();
    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        gtk::Inhibit(false)
    });


    let mut app = TaskTreeState {
        treestore: builder.get_object("tasktree").unwrap(),
        filterbuffer: builder.get_object("filterbuffer").unwrap(),
        tasks: std::collections::HashMap::new(),
    };
    app.refresh();
    window.show_all();
    gtk::main();
}
