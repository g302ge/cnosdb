use async_trait::async_trait;

use spi::query::execution::{Output, QueryExecution, QueryStateMachineRef};
use spi::query::logical_planner::DDLPlan;
use spi::query::{self, QueryError};

use spi::query::execution::ExecutionError;

use self::create_table::CreateTableTask;
use crate::execution::ddl::create_database::CreateDatabaseTask;
use crate::execution::ddl::describe_database::DescribeDatabaseTask;
use crate::execution::ddl::describe_table::DescribeTableTask;
use crate::execution::ddl::show_database::ShowDatabasesTask;
use crate::execution::ddl::show_table::ShowTablesTask;
use snafu::ResultExt;

use self::create_external_table::CreateExternalTableTask;
use self::drop_object::DropObjectTask;

mod create_database;
mod create_external_table;
mod create_table;
mod describe_database;
mod describe_table;
mod drop_object;
mod show_database;
mod show_table;

/// Traits that DDL tasks should implement
#[async_trait]
trait DDLDefinitionTask: Send + Sync {
    async fn execute(
        &self,
        query_state_machine: QueryStateMachineRef,
    ) -> Result<Output, ExecutionError>;
}

pub struct DDLExecution {
    task_factory: DDLDefinitionTaskFactory,
    query_state_machine: QueryStateMachineRef,
}

impl DDLExecution {
    pub fn new(query_state_machine: QueryStateMachineRef, plan: DDLPlan) -> Self {
        Self {
            task_factory: DDLDefinitionTaskFactory { plan },
            query_state_machine,
        }
    }
}

#[async_trait]
impl QueryExecution for DDLExecution {
    // execute ddl task
    // This logic usually does not change
    async fn start(&self) -> Result<Output, QueryError> {
        let query_state_machine = self.query_state_machine.clone();

        self.task_factory
            .create_task()
            .execute(query_state_machine)
            .await
            .context(query::ExecutionSnafu)
    }
}

struct DDLDefinitionTaskFactory {
    plan: DDLPlan,
}

impl DDLDefinitionTaskFactory {
    // According to different statement types, construct the corresponding task
    // If you add ddl operations, you usually need to modify here
    fn create_task(&self) -> Box<dyn DDLDefinitionTask> {
        match &self.plan {
            DDLPlan::CreateExternalTable(sub_plan) => {
                Box::new(CreateExternalTableTask::new(sub_plan.clone()))
            }
            DDLPlan::Drop(sub_plan) => Box::new(DropObjectTask::new(sub_plan.clone())),
            DDLPlan::CreateTable(sub_plan) => Box::new(CreateTableTask::new(sub_plan.clone())),
            DDLPlan::CreateDatabase(sub_plan) => {
                Box::new(CreateDatabaseTask::new(sub_plan.clone()))
            }
            DDLPlan::DescribeDatabase(sub_plan) => {
                Box::new(DescribeDatabaseTask::new(sub_plan.clone()))
            }
            DDLPlan::DescribeTable(sub_plan) => Box::new(DescribeTableTask::new(sub_plan.clone())),
            DDLPlan::ShowTables(sub_plan) => Box::new(ShowTablesTask::new(sub_plan.clone())),
            DDLPlan::ShowDatabases() => Box::new(ShowDatabasesTask::new()),
        }
    }
}
