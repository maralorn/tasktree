use uuid;
use task_hookrs::*;
use serde_json;
use util;
use std;
use regex;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Task {
    pub status: status::TaskStatus,
    pub uuid: uuid::Uuid,
    pub entry: date::Date,
    pub description: String,

    pub partof: Option<uuid::Uuid>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<Vec<annotation::Annotation>>,

    // #[serde(skip_serializing_if = "Option::is_none")]
    // pub depends     : Option<String>,
    //
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due: Option<date::Date>,

    // #[serde(skip_serializing_if = "Option::is_none")]
    // pub end         : Option<Date>,
    //
    // #[serde(skip_serializing_if = "Option::is_none")]
    // pub imask       : Option<i64>,
    //
    // #[serde(skip_serializing_if = "Option::is_none")]
    // pub mask        : Option<String>,
    //
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified: Option<date::Date>,

    // #[serde(skip_serializing_if = "Option::is_none")]
    // pub parent      : Option<Uuid>,
    //
    // #[serde(skip_serializing_if = "Option::is_none")]
    // pub priority    : Option<TaskPriority>,
    //
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project: Option<project::Project>,

    // #[serde(skip_serializing_if = "Option::is_none")]
    // pub recur       : Option<String>,
    //
    // #[serde(skip_serializing_if = "Option::is_none")]
    // pub scheduled   : Option<Date>,
    //
    // #[serde(skip_serializing_if = "Option::is_none")]
    // pub start       : Option<Date>,
    //
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<tag::Tag>>,

    // #[serde(skip_serializing_if = "Option::is_none")]
    // pub until       : Option<Date>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wait: Option<date::Date>,
}

pub struct TaskCache {
    tasks: std::collections::HashMap<uuid::Uuid, Task>,
}
impl TaskCache {
    pub fn new() -> Self {
        TaskCache { tasks: std::collections::HashMap::new() }
    }
    pub fn create(
        &mut self,
        description: String,
        partof: Option<&uuid::Uuid>,
    ) -> util::Result<&Task> {
        lazy_static! {
            static ref UUID_RE: regex::Regex = regex::Regex::new("[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}").unwrap();
        }
        let mut command = std::process::Command::new("task");
        command
            .stdout(std::process::Stdio::piped())
            .arg("add")
            .arg("rc.verbose=new-uuid")
            .arg(description);
        if let Some(uuid) = partof {
            command.arg(format!("partof:{}", uuid));
        };
        let stdout = command.output()?.stdout;
        let uuid_match = UUID_RE
            .captures_iter(std::str::from_utf8(&stdout)?)
            .next()
            .ok_or("No uuid in task feedback found")?;
        self.update(&uuid::Uuid::parse_str(&uuid_match[0])?)
    }
    pub fn refresh(&mut self) -> util::Result<()> {
        let stdout = &std::process::Command::new("task")
            .arg("export")
            .stdout(std::process::Stdio::piped())
            .output()?
            .stdout;
        let tasks: Vec<Task> = serde_json::from_str(std::str::from_utf8(stdout)?)?;
        for task in tasks {
            self.tasks.insert(task.uuid, task);
        }
        Ok(())
    }
    pub fn get_task(&self, uuid: &uuid::Uuid) -> util::Result<&Task> {
        Ok(self.tasks.get(uuid).ok_or("Uuid not found in Cache")?)
    }
    pub fn update(&mut self, uuid: &uuid::Uuid) -> util::Result<&Task> {
        let stdout = &std::process::Command::new("task")
            .arg("export")
            .arg(format!("uuid:{}", uuid))
            .stdout(std::process::Stdio::piped())
            .output()?
            .stdout;
        let tasks: Vec<Task> = serde_json::from_str(std::str::from_utf8(stdout)?)?;
        for task in tasks {
            self.tasks.insert(*uuid, task);
            return Ok(self.tasks.get(uuid).unwrap());
        }
        Err("Could not load Task!")?
    }
}
pub fn get_tasks(query: &str) -> util::Result<Vec<uuid::Uuid>> {
    let stdout = &std::process::Command::new("task")
        .arg("_uuid")
        .arg(query)
        .output()?
        .stdout;
    let mut tasks = vec![];
    for uuid_str in std::str::from_utf8(stdout)?.split_whitespace() {
        tasks.push(uuid::Uuid::parse_str(uuid_str)?)
    }
    Ok(tasks)
}
pub fn done(uuid: &uuid::Uuid) -> util::Result<()> {
    &std::process::Command::new("task")
        .arg(format!("uuid:{}", uuid))
        .arg("done")
        .output()?;
    Ok(())
}
pub fn delete(uuid: &uuid::Uuid) -> util::Result<()> {
    &std::process::Command::new("task")
        .arg(format!("uuid:{}", uuid))
        .arg("delete")
        .arg("rc.confirmation:0")
        .output()?;
    Ok(())
}
pub fn pending(uuid: &uuid::Uuid) -> util::Result<()> {
    &std::process::Command::new("task")
        .arg(format!("uuid:{}", uuid))
        .arg("mod")
        .arg("status:pending")
        .output()?;
    Ok(())
}
pub fn partof(uuid: &uuid::Uuid, partof: Option<&uuid::Uuid>) -> util::Result<()> {
    &std::process::Command::new("task")
        .arg(format!("uuid:{}", uuid))
        .arg("mod")
        .arg(format!(
            "partof:{}",
            partof.map_or("".to_string(), ToString::to_string)
        ))
        .output()?;
    Ok(())
}
