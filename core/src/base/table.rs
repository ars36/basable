use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use serde::{Deserialize, Serialize};

use crate::base::column::ColumnList;

use super::ConnectorType;

pub(crate) type SharedTable<E, R, C> = Arc<Mutex<dyn Table<Error = E, Row = R, ColumnValue = C>>>;

pub(crate) type TableSummaries = Vec<TableSummary>;
pub(crate) type TableConfigs = Option<Vec<TableConfig>>;

pub(crate) type DataQueryResult<V, E> = Result<Vec<HashMap<String, V>>, E>;

/// Table column used for querying table history such as when a row was added or when a row was updated.
#[derive(Deserialize, Serialize, Clone)]
pub(crate) struct HistoryColumn {
    name: String,
    format: String,
    has_time: bool,
}

/// The type of `SpecialColumn`
#[derive(Deserialize, Serialize, Clone)]
pub(crate) enum SpecialValueType {
    Image,
    Audio,
    Video,
    PDF,
    Webpage,
}

/// Special columns are columns whose values should lead to some sort of media types.
#[derive(Deserialize, Serialize, Clone)]
pub(crate) struct SpecialColumn {
    name: String,
    special_type: SpecialValueType,
    path: String,
}

/// The action that should trigger `NotifyEvent`.
#[derive(Deserialize, Serialize, Clone)]
enum NotifyTrigger {
    Create,
    Update,
    Delete,
}

/// When should `NotifyEvent` get triggered around `NotifyTrigger`.
#[derive(Deserialize, Serialize, Clone)]
pub(crate) enum NotifyTriggerTime {
    Before,
    After,
}

/// The REST API method expected by the webhook URL.
#[derive(Deserialize, Serialize, Clone)]
pub(crate) enum NotifyEventMethod {
    Get,
    Post,
    Delete,
    Put,
    Patch,
}

/// What should happen to the operation `NotifyTrigger` when there's notification error?
/// Let's say there's a server error from the webhook URL, should we proceed or fail the operation?
#[derive(Deserialize, Serialize, Clone)]
pub(crate) enum OnNotifyError {
    Fail,
    Proceed,
}

/// Event sent to a given webhook URL based on certain `NotifyTrigger`
#[derive(Deserialize, Serialize, Clone)]
pub(crate) struct NotifyEvent {
    trigger: NotifyTrigger,
    trigger_time: NotifyTriggerTime,
    method: NotifyEventMethod,
    url: String,
    on_error: OnNotifyError,
}

#[derive(Deserialize, Serialize, Clone)]
pub(crate) struct TableConfig {
    pub table_id: String,

    /// Name of column to use as primary key.
    pub pk: Option<String>,

    /// Column for querying when a row was created.
    pub created_column: Option<HistoryColumn>,

    /// Column for querying when a row was updated.
    pub updated_column: Option<HistoryColumn>,

    /// Special columns that return `SpecialValueType`
    pub special_columns: Option<Vec<SpecialColumn>>,

    /// Notification events for this table.
    pub events: Option<Vec<NotifyEvent>>,
}

impl PartialEq for TableConfig {
    fn eq(&self, other: &Self) -> bool {
        self.table_id == other.table_id
    }
}

impl Default for TableConfig {
    fn default() -> Self {
        TableConfig {
            pk: None,
            table_id: String::new(),
            created_column: None,
            updated_column: None,
            special_columns: None,
            events: None,
        }
    }
}

pub struct DataQueryFilter {
    /// Query pagination
    pub limit: usize,

    /// Columns to exclude from query
    pub exclude: Option<Vec<String>>,
}

impl Default for DataQueryFilter {
    fn default() -> Self {
        DataQueryFilter {
            limit: 100,
            exclude: None,
        }
    }
}

#[derive(Serialize)]
pub(crate) struct TableSummary {
    pub name: String,
    pub row_count: u32,
    pub col_count: u32,
    pub created: Option<String>,
    pub updated: Option<String>,
}

#[derive(Deserialize, Default)]
pub(crate) struct UpdateDataOptions {
    pub key: String,
    pub value: String,
    pub input: HashMap<String, String>,
}

pub(crate) trait Table: Sync + Send {
    type Error;
    type Row;
    type ColumnValue;

    /// Create a new [`Table`] and assign the given [`ConnectorType`].
    ///
    /// It creates new [`Table`] and returns a [`TableConfig`]  for the table when possible.
    fn new(name: String, conn: ConnectorType) -> (Self, Option<TableConfig>)
    where
        Self: Sized;

    /// [Table]'s name
    fn name(&self) -> &str;

    /// Get the table's [`ConnectorType`].
    fn connector(&self) -> &ConnectorType;

    /// Retrieve all columns for the table
    fn query_columns(&self) -> Result<ColumnList, Self::Error>;

    /// Inserts a new data into the table.
    fn insert_data(&self, input: HashMap<String, String>) -> Result<(), Self::Error>;

    /// Retrieve data from table based on query `filter`.
    fn query_data(
        &self,
        filter: DataQueryFilter,
    ) -> DataQueryResult<Self::ColumnValue, Self::Error>;

    fn update_data(&self, input: UpdateDataOptions) -> Result<(), Self::Error>;
}

#[cfg(test)]
mod tests {

    use std::{collections::HashMap, io::stdin};

    use crate::{
        base::{
            table::{DataQueryFilter, UpdateDataOptions},
            AppError,
        },
        tests::common::{create_test_instance, get_test_db_table, get_test_user_id},
    };

    #[test]
    fn test_table_exist() -> Result<(), AppError> {
        let user_id = get_test_user_id();
        let bsbl = create_test_instance(true)?;

        let user = bsbl.find_user(&user_id);
        let user = user.unwrap().borrow();

        let db = user.db().unwrap();
        // let db_ref = db.clone();
        let db = db.lock().unwrap();

        let table_name = get_test_db_table();

        // let tt = Arc::new(Box::new(db_ref));
        // db.load_tables(db_ref)?;
        assert!(db.table_exists(&table_name)?);

        Ok(())
    }

    #[test]
    fn test_table_query_column() -> Result<(), AppError> {
        let user_id = get_test_user_id();
        let bsbl = create_test_instance(true)?;

        let user = bsbl.find_user(&user_id);
        let user = user.unwrap().borrow();

        let db = user.db();
        let db = db.unwrap();
        let db = db.lock().unwrap();

        let table_name = get_test_db_table();

        assert!(db.get_table(&table_name).is_some());

        if let Some(table) = db.get_table("swp") {
            let table = table.lock().unwrap();
            let cols = table.query_columns();

            assert!(cols.is_ok());
        }

        Ok(())
    }

    #[test]
    fn test_table_query_data() -> Result<(), AppError> {
        let user_id = get_test_user_id();
        let bsbl = create_test_instance(true)?;

        let user = bsbl.find_user(&user_id);
        let user = user.unwrap().borrow();

        let db = user.db();
        let db = db.unwrap();
        let db = db.lock().unwrap();

        let table_name = get_test_db_table();

        if let Some(table) = db.get_table(&table_name) {
            let table = table.lock().unwrap();
            let filter = DataQueryFilter::default();
            let data = table.query_data(filter);
            assert!(data.is_ok());
        }

        Ok(())
    }

    #[test]
    fn test_table_insert_data() -> Result<(), AppError> {
        let user_id = get_test_user_id();
        let bsbl = create_test_instance(true)?;

        let user = bsbl.find_user(&user_id);
        let user = user.unwrap().borrow();

        let db = user.db();
        let db = db.unwrap();
        let db = db.lock().unwrap();

        let table_name = get_test_db_table();

        if let Some(table) = db.get_table(&table_name) {
            let mut test_data = HashMap::new();
            let quit_word = "quit";

            println!(
                "
                Let's add some data into our TEST_DB_TABLE_NAME. \n
                Please enter your data inputs in the format: column,value. \n
                Enter '{}' to quit the program.
            ",
                quit_word
            );

            loop {
                let mut input = String::new();
                println!("Please enter an input:");
                stdin()
                    .read_line(&mut input)
                    .expect("Please enter a valid string");

                let input = input.trim().to_string();
                if input == quit_word {
                    break;
                }

                let spl: Vec<&str> = input.split(",").collect();
                test_data.insert(spl[0].to_string(), spl[1].to_string());
            }

            let table = table.lock().unwrap();
            let insert_data = table.insert_data(test_data);

            assert!(insert_data.is_ok());
        }

        Ok(())
    }

    #[test]
    fn test_table_update_data() -> Result<(), AppError> {
        let user_id = get_test_user_id();
        let bsbl = create_test_instance(true)?;

        let user = bsbl.find_user(&user_id);
        let user = user.unwrap().borrow();

        let db = user.db();
        let db = db.unwrap();
        let db = db.lock().unwrap();

        let table_name = get_test_db_table();

        if let Some(table) = db.get_table(&table_name) {
            let mut test_data = UpdateDataOptions::default();
            
            // Get update clause 
            println!("Please enter update clause as key,value");
            let mut input = String::new();

            stdin()
                .read_line(&mut input)
                .expect("Please enter a valid string");

            let input = input.trim().to_string();

            let spl: Vec<&str> = input.split(",").collect();
            test_data.key = spl[0].to_string();
            test_data.value = spl[1].to_string();

            // Get update value
            println!("Please enter update clause as column,value");
            let mut input = String::new();

            stdin()
                .read_line(&mut input)
                .expect("Please enter a valid string");

            let input = input.trim().to_string();

            let spl: Vec<&str> = input.split(",").collect();
            test_data.input.insert(spl[0].to_string(), spl[1].to_string());

            // update the table
            let table = table.lock().unwrap();
            let update_data = table.update_data(test_data);

            assert!(update_data.is_ok());
        }

        Ok(())
    }
}