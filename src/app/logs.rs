use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicI64, Ordering},
        Arc,
    },
};

use rust_extensions::date_time::DateTimeAsMicroseconds;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Copy)]
pub enum SystemProcess {
    System = 0,
    TcpSocket = 1,
    PersistOperation = 2,
    TableOperation = 3,
    Init = 4,
    Timer = 5,
    Debug = 6,
}

impl SystemProcess {
    pub fn iterate() -> Vec<Self> {
        vec![
            SystemProcess::System,
            SystemProcess::TcpSocket,
            SystemProcess::PersistOperation,
            SystemProcess::TableOperation,
            SystemProcess::Init,
            SystemProcess::Timer,
        ]
    }
    pub fn parse(value: &str) -> Option<Self> {
        let value = value.to_ascii_lowercase();
        if value == "system" {
            return Some(SystemProcess::System);
        }

        if value == "tcpsocket" {
            return Some(SystemProcess::TcpSocket);
        }

        if value == "persistoperation" {
            return Some(SystemProcess::PersistOperation);
        }

        if value == "tableoperation" {
            return Some(SystemProcess::TableOperation);
        }

        if value == "init" {
            return Some(SystemProcess::Init);
        }

        if value == "timer" {
            return Some(SystemProcess::Timer);
        }

        return None;
    }

    pub fn as_u8(&self) -> u8 {
        let result = *self as u8;
        return result;
    }
}

#[derive(Debug, Clone)]
pub enum LogLevel {
    Info,
    Error,
    FatalError,
}

impl LogLevel {
    pub fn is_fatal_error(&self) -> bool {
        match self {
            LogLevel::FatalError => true,
            _ => false,
        }
    }
}
#[derive(Debug, Clone)]
pub struct LogItem {
    pub date: DateTimeAsMicroseconds,

    pub table: Option<String>,

    pub level: LogLevel,

    pub process: SystemProcess,

    pub process_name: String,

    pub message: String,

    pub err_ctx: Option<HashMap<String, String>>,
}

struct LogsData {
    items: Vec<Arc<LogItem>>,
    items_by_table: HashMap<String, Vec<Arc<LogItem>>>,
    items_by_process: HashMap<u8, Vec<Arc<LogItem>>>,
    fatal_errors: Vec<Arc<LogItem>>,
}

pub struct Logs {
    data: Arc<RwLock<LogsData>>,
    fatal_errors_amount: AtomicI64,
}

impl Logs {
    pub fn new() -> Self {
        let logs_data = LogsData {
            items: Vec::new(),
            items_by_table: HashMap::new(),
            items_by_process: HashMap::new(),
            fatal_errors: Vec::new(),
        };

        Self {
            data: Arc::new(RwLock::new(logs_data)),
            fatal_errors_amount: AtomicI64::new(0),
        }
    }

    pub fn add_info(
        &self,
        table: Option<String>,
        process: SystemProcess,
        process_name: String,
        message: String,
        context: Option<HashMap<String, String>>,
    ) {
        let logs_data = self.data.clone();
        tokio::spawn(async move {
            let item = LogItem {
                date: DateTimeAsMicroseconds::now(),
                level: LogLevel::Info,
                table,
                process_name,
                process,
                message: message,
                err_ctx: context,
            };

            add(logs_data, item).await;
        });
    }

    pub fn get_fatal_errors_amount(&self) -> i64 {
        self.fatal_errors_amount.load(Ordering::Relaxed)
    }

    pub async fn get_fatal_errors(&self) -> Option<Vec<Arc<LogItem>>> {
        let data = self.data.read().await;
        if data.fatal_errors.is_empty() {
            return None;
        }

        let result = data.fatal_errors.clone();
        return result.into();
    }

    pub fn add_error(
        &self,
        table: Option<String>,
        process: SystemProcess,
        process_name: String,
        message: String,
        err_ctx: Option<HashMap<String, String>>,
    ) {
        let logs_data = self.data.clone();

        tokio::spawn(async move {
            let item = LogItem {
                date: DateTimeAsMicroseconds::now(),
                level: LogLevel::Error,
                table,
                process_name,
                process,
                message: message,
                err_ctx,
            };

            add(logs_data, item).await;
        });
    }

    pub fn add_fatal_error(
        &self,
        table: Option<String>,
        process: SystemProcess,
        process_name: String,
        message: String,
        context: Option<HashMap<String, String>>,
    ) {
        let logs_data = self.data.clone();
        self.fatal_errors_amount.fetch_add(1, Ordering::SeqCst);

        tokio::spawn(async move {
            let item = LogItem {
                date: DateTimeAsMicroseconds::now(),
                level: LogLevel::FatalError,
                table,
                process_name,
                process,
                message: message,
                err_ctx: context,
            };

            add(logs_data, item).await;
        });
    }

    pub async fn get(&self) -> Vec<Arc<LogItem>> {
        let read_access = self.data.read().await;
        read_access.items.to_vec()
    }

    pub async fn get_by_table_name(&self, table_name: &str) -> Option<Vec<Arc<LogItem>>> {
        let read_access = self.data.read().await;
        let result = read_access.items_by_table.get(table_name)?;
        return Some(result.to_vec());
    }

    pub async fn get_by_process(&self, process: SystemProcess) -> Option<Vec<Arc<LogItem>>> {
        let read_access = self.data.read().await;
        let result = read_access.items_by_process.get(&process.as_u8())?;
        return Some(result.to_vec());
    }
}

fn print_to_console(item: &LogItem) {
    println!("----------");
    println!(
        "{} {:?} {:?}",
        item.date.to_rfc3339(),
        item.level,
        item.process
    );
    if let Some(table) = &item.table {
        println!("Table: {}", table);
    }
    println!("Process: {}", item.process_name);
    println!("Message: {}", item.message);
    if let Some(err_ctx) = &item.err_ctx {
        println!("Err_ctx: {:?}", err_ctx);
    }
}

fn add_table_data<T>(
    items_by_table: &mut HashMap<T, Vec<Arc<LogItem>>>,
    category: &T,
    item: Arc<LogItem>,
) where
    T: Eq + std::hash::Hash + Clone + Sized,
{
    if !items_by_table.contains_key(category) {
        items_by_table.insert(category.clone(), Vec::new());
    }

    let items = items_by_table.get_mut(category).unwrap();

    items.push(item);

    gc_logs(items);
}

fn gc_logs(items: &mut Vec<Arc<LogItem>>) {
    while items.len() > 100 {
        items.remove(0);
    }
}

fn should_log_be_printed(item: &LogItem) -> bool {
    match &item.level {
        LogLevel::Info => {}
        LogLevel::Error => {
            return true;
        }
        LogLevel::FatalError => {
            return true;
        }
    }

    match &item.process {
        SystemProcess::System => {
            return true;
        }
        SystemProcess::TcpSocket => {}
        SystemProcess::PersistOperation => {}
        SystemProcess::TableOperation => {}
        SystemProcess::Init => {
            return true;
        }
        SystemProcess::Debug => {
            return true;
        }
        SystemProcess::Timer => {}
    }

    return false;
}

async fn add(logs_data: Arc<RwLock<LogsData>>, item: LogItem) {
    let item = Arc::new(item);
    let mut wirte_access = logs_data.write().await;

    if should_log_be_printed(&item) {
        print_to_console(&item);
    }

    if item.level.is_fatal_error() {
        wirte_access.fatal_errors.push(item.clone());
        while wirte_access.fatal_errors.len() > 100 {
            wirte_access.fatal_errors.remove(0);
        }
    }

    let process_id = item.as_ref().process.as_u8();

    add_table_data(
        &mut wirte_access.items_by_process,
        &process_id,
        item.clone(),
    );

    if let Some(table_name) = &item.table {
        add_table_data(&mut wirte_access.items_by_table, table_name, item.clone());
    }

    let items = &mut wirte_access.items;
    items.push(item);
    gc_logs(items);
}
