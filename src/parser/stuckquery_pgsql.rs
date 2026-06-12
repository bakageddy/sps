use std::net::Ipv4Addr;
use time::{Date, PrimitiveDateTime, Time, UtcDateTime, macros::format_description};
use tracing::warn;

use crate::{error::stuckquery::PgParse, parser::scanner::Scanner, util::ToUnixMillis};

#[derive(Debug)]
pub struct StuckQueryTable<'a> {
    pub queries: Vec<StuckQuery<'a>>,
    pub timestamp: u64,
}

#[derive(Debug)]
pub struct StuckQuery<'a> {
    pub db_name: &'a str,
    pub query: &'a str,
    pub application_name: &'a str,
    pub client_hostname: Option<&'a str>,
    pub pid: u64,
    pub query_time_ms: u64,
    pub txn_time_ms: u64,
    pub last_state_change: u64,
    pub client_address: Option<Ipv4Addr>,
    pub client_port: Option<u16>,
    pub state: State,
    pub waiting: bool,
}

static STATE_CHANGE_FORMAT: &[time::format_description::FormatItem<'static>] = format_description!(
    "[year]-[month]-[day] [hour]:[minute]:[second].[subsecond][offset_hour sign:mandatory]:[offset_minute]"
);

pub static TIMESTAMP_TIME_FORMAT: &[time::format_description::FormatItem<'static>] =
    format_description!("[hour]:[minute]:[second].[subsecond]");

pub static TIMESTAMP_DATE_FORMAT: &[time::format_description::FormatItem<'static>] =
    format_description!("[day]-[month]-[year]");

#[derive(Debug)]
pub enum State {
    Active,
    Idle,
}

impl<'a> StuckQueryTable<'a> {
    pub fn extract_timestamp(header: &'a str) -> Result<u64, PgParse> {
        let mut scanner = Scanner::new(header);
        let time = scanner
            .take_within("[", "]")
            .map_err(|_| PgParse::TableHeaderMetaExtraction)?;
        let date = scanner
            .take_within("[", "]")
            .map_err(|_| PgParse::TableHeaderMetaExtraction)?;
        let parsed_time =
            Time::parse(time, TIMESTAMP_TIME_FORMAT).map_err(PgParse::TimestampParse)?;
        let parsed_date =
            Date::parse(date, TIMESTAMP_DATE_FORMAT).map_err(PgParse::TimestampParse)?;
        let datetime = PrimitiveDateTime::new(parsed_date, parsed_time);
        Ok(datetime.to_unix_millis().unwrap_or(0))
    }
}

impl<'a> TryFrom<&'a str> for StuckQueryTable<'a> {
    type Error = PgParse;
    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        let mut scanner = Scanner::new(value);
        let mut queries = Vec::new();
        let header = scanner
            .take_until("\n")
            .ok_or_else(|| PgParse::TableHeaderExtraction)?;

        let timestamp = StuckQueryTable::extract_timestamp(header)?;

        for _ in 0..7 {
            let line = scanner.take_until("\n");
            if line.is_none() {
                warn!("Table is empty, skipping parsing");
                return Err(PgParse::TableExtraction);
            }
        }

        while let Some(line) = scanner.take_until("\n") {
            if !line.trim().starts_with("|") {
                break;
            }
            match StuckQuery::try_from(line) {
                Ok(x) => queries.push(x),
                Err(e) => warn!("Cannot parse {line} due to {e:?}"),
            }
        }
        Ok(Self { queries, timestamp })
    }
}

impl<'a> TryFrom<&'a str> for StuckQuery<'a> {
    type Error = PgParse;

    // TODO: Migrate to scanner
    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        let mut iter = value.split("|");
        iter.next().ok_or_else(|| PgParse::EmptyBlock)?;
        let pid = iter
            .next()
            .ok_or_else(|| PgParse::PidExtraction)?
            .trim()
            .parse()
            .map_err(PgParse::InvalidPID)?;
        let query_time = iter
            .next()
            .ok_or_else(|| PgParse::QueryTimeExtraction)?
            .trim()
            .parse::<f32>()
            .map_err(PgParse::InvalidQueryTime)?;
        let query_time_ms = (query_time * 1000.0f32).trunc() as u64;

        let txn_time = iter
            .next()
            .ok_or_else(|| PgParse::TransactionTimeExtraction)?
            .trim();
        let txn_time = if txn_time.is_empty() {
            Ok(0.0)
        } else {
            txn_time.parse().map_err(PgParse::InvalidTransactionTime)
        }?;
        let txn_time_ms = (txn_time * 1000.0f32).trunc() as u64;

        let db_name = iter
            .next()
            .ok_or_else(|| PgParse::DatabaseNameExtraction)?
            .trim();
        let state = iter.next().ok_or_else(|| PgParse::StateExtraction)?.trim();
        let state = State::try_from(state)?;

        let waiting = iter
            .next()
            .ok_or_else(|| PgParse::WaitingExtraction)?
            .trim();
        let waiting = match waiting {
            "f" => false,
            "t" => true,
            _ => {
                return Err(PgParse::InvalidWaitingState {
                    got: String::from(waiting),
                });
            }
        };

        let query = iter.next().ok_or_else(|| PgParse::QueryExtraction)?.trim();

        let state_change = iter.next().ok_or(PgParse::StateChangeExtraction)?.trim();
        let state_change = UtcDateTime::parse(state_change, STATE_CHANGE_FORMAT)
            .map_err(PgParse::InvalidStateChange)?
            .unix_timestamp_nanos()
            / 1_000_000;
        let state_change = state_change as u64;

        let application_name = iter
            .next()
            .ok_or(PgParse::ApplicationNameExtraction)?
            .trim();

        let client_address = iter.next().ok_or(PgParse::ClientAddressExtraction)?.trim();
        let client_address = if client_address.is_empty() {
            None
        } else {
            Some(client_address.parse()?)
        };

        let client_hostname = iter.next().ok_or(PgParse::ClientHostnameExtraction)?;
        let client_hostname = if client_hostname.trim().is_empty() {
            None
        } else {
            Some(client_hostname)
        };

        let client_port = iter.next().ok_or(PgParse::ClientPortExtraction)?.trim();
        let client_port = if !client_port.is_empty() {
            Some(client_port.parse().map_err(PgParse::InvalidClientPort)?)
        } else {
            None
        };

        Ok(StuckQuery {
            pid,
            client_port,
            client_hostname,
            db_name,
            application_name,
            state,
            waiting,
            client_address,
            query_time_ms,
            query,
            txn_time_ms,
            last_state_change: state_change,
        })
    }
}

impl State {
    pub fn is_active(&self) -> bool {
        if let Self::Active = self {
            return true;
        } else {
            false
        }
    }
}

impl TryFrom<&str> for State {
    type Error = PgParse;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "active" => Ok(State::Active),
            "idle in transaction" => Ok(State::Idle),
            _ => Err(PgParse::UnrecognizedState(String::from(value))),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{
        ingest::stuckquery_pgsql::StuckQueryTableIteratorPGSQL,
        parser::stuckquery_pgsql::{StuckQuery, StuckQueryTable},
        util,
    };
    #[test]
    fn stuckquery_pgsql_parse_single_row_no_client_info() {
        let input = "|  11532  |  4542.352065     |  4542.359728   |  servicedesk  |  active               |  f        |  SELECT count(*) as orphanentries FROM  notes  WHERE	 (notesid  not in 	(  	SELECT notesid 	FROM  workordernotes  	))  limit 50000                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                               |  2026-02-18 15:02:14.229876+05:30  |  PostgreSQL JDBC Driver  |                  |                   |               |";
        let output = StuckQuery::try_from(input);
        assert!(output.is_ok(), "{}", output.unwrap_err());
        let output = output.unwrap();
        println!("output: {output:#?}");
    }

    #[test]
    fn stuckquery_pgsql_parse_single_row_with_client_info() {
        let input = "|  16860  |  1931.755461     |  1931.766792   |  servicedesk  |  active               |  f        |  SELECT count(*) as orphanentries FROM  notes  WHERE	 (notesid  not in 	(  	SELECT notesid 	FROM  workordernotes  	))  limit 50000                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                               |  2026-02-18 15:45:44.739882+05:30  |  PostgreSQL JDBC Driver  |  127.0.0.1       |                   |  60320        |";
        let output = StuckQuery::try_from(input);
        assert!(output.is_ok(), "{}", output.unwrap_err());
        let output = output.unwrap();
        println!("output: {output:#?}");
    }

    #[test]
    fn stuckquery_pgsql_parse_single_row_negative_timestamp() {
        let input = "|  13280  |  -0.001069       |  -0.001069     |  servicedesk  |  active  |  f        |  SELECT PurchaseOrder.PURCHASEORDERID AS \"PurchaseOrderID\", PurchaseOrder.POCUSTOMID AS \"POCustomID\", PurchaseOrder.PONAME AS \"POName\", PurchaseOrder.OWNERID, PurchaseOrder.DATEORDERED, AaaUser.USER_ID, AaaUser.FIRST_NAME AS \"Owner\", PurchaseOrder.DATEREQUIRED AS \"Required By\", PurchaseOrder.STATUSID, POStatus.STATUSID, POStatus.STATUSNAME AS \"Status\" FROM PurchaseOrder INNER JOIN POStatus ON PurchaseOrder.STATUSID=POStatus.STATUSID LEFT JOIN AaaUser ON PurchaseOrder.OWNERID=AaaUser.USER_ID LEFT JOIN WorkOrderToPurchaseOrder ON PurchaseOrder.PURCHASEORDERID=WorkOrderToPurchaseOrder.PURCHASEORDERID WHERE  (( PurchaseOrder.HELPDESKID = 1 ) AND ( WorkOrderToPurchaseOrder.WORKORDERID = 3485639 ))  ORDER BY 5 DESC LIMIT 25  |  2026-02-18 16:22:19.2058+05:30    |  PostgreSQL JDBC Driver  |  127.0.0.1       |                   |  58517        |";
        let output = StuckQuery::try_from(input);
        assert!(output.is_ok(), "{}", output.unwrap_err());
        let output = output.unwrap();
        println!("output: {output:#?}");
    }

    #[test]
    fn stuckquery_pgsql_parse_table() {
        let map = util::map_file("test/stuckquery_pgsql_single_table.txt").unwrap();
        let table = StuckQueryTableIteratorPGSQL(&map).next();
        assert!(table.is_some(), "Failed to stream tables");
        let table = table.unwrap();
        let table = StuckQueryTable::try_from(table);
        assert!(table.is_ok(), "Error: {:?}", table.unwrap_err());
        let table = table.unwrap();
        assert_eq!(table.queries.len(), 24);
    }

    #[test]
    fn stuckquery_pgsql_parse_full_file() {
        let map = util::map_file("test/stuckqueries_pgsql_full.txt").unwrap();
        for table in StuckQueryTableIteratorPGSQL(&map) {
            let table = StuckQueryTable::try_from(table);
            assert!(table.is_ok(), "Error: {:?}", table.unwrap_err());
            let table = table.unwrap();
            assert_ne!(table.queries.len(), 0);
        }
    }

    #[test]
    fn stuckquery_pgsql_parse_empty_table() {
        let map = util::map_file("test/stuckquery_pgsql_empty_table.txt").unwrap();
        for table in StuckQueryTableIteratorPGSQL(&map) {
            let table = StuckQueryTable::try_from(table);
            assert!(table.is_ok(), "Error: {:?}", table.unwrap_err());
            let table = table.unwrap();
            assert_eq!(table.queries.len(), 0);
        }
    }
}
