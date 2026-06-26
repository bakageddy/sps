use std::str::FromStr;

use crate::util::ToUnixMillis;
use crate::{
    error::cpumonitoring::Parse,
    parser::{scanner::Scanner, stacktrace::Trace},
};
use time::{
    Date, PrimitiveDateTime, Time, format_description::FormatItem, macros::format_description,
};

static CPU_MONITORING_TIME_FORMAT: &[FormatItem<'static>] =
    format_description!("[hour]:[minute]:[second].[subsecond]");

static CPU_MONITORING_DATE_FORMAT: &[FormatItem<'static>] =
    format_description!("[day]-[month]-[year]");

#[derive(Debug)]
pub struct CPUThread<'a> {
    pub name: &'a str,
    pub trace: Option<Trace<'a>>,
    pub timestamp: u64,
    pub tid: u64,
    pub cpu: f32,
    pub state: State,
}

#[derive(Debug, PartialEq, Eq)]
pub enum State {
    Runnable,
    TimedWaiting,
    Blocked,
    Waiting,
}

impl FromStr for State {
    type Err = Parse;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "RUNNABLE" => Ok(State::Runnable),
            "TIMED_WAITING" => Ok(State::TimedWaiting),
            "BLOCKED" => Ok(State::Blocked),
            "WAITING" => Ok(State::Waiting),
            s => Err(Parse::UnrecognizedState(String::from(s))),
        }
    }
}

impl State {
    pub fn to_str(&self) -> &'static str {
        match self {
            Self::Runnable => "RUNNABLE",
            Self::Blocked => "BLOCKED",
            Self::TimedWaiting => "TIMED_WAITING",
            Self::Waiting => "WAITING",
        }
    }
}

impl<'a> CPUThread<'a> {
    fn extract_timestamp(header: &str) -> Result<u64, Parse> {
        let mut scanner = Scanner::new(header);
        let time = scanner
            .take_within("[", "]")
            .map_err(|_| Parse::MonitoringHeaderExtraction)?;
        let date = scanner
            .take_within("[", "]")
            .map_err(|_| Parse::MonitoringHeaderExtraction)?;

        let parsed_time =
            Time::parse(time, CPU_MONITORING_TIME_FORMAT).map_err(Parse::MonitoringHeaderParse)?;

        let parsed_date =
            Date::parse(date, CPU_MONITORING_DATE_FORMAT).map_err(Parse::MonitoringHeaderParse)?;

        Ok(PrimitiveDateTime::new(parsed_date, parsed_time)
            .to_unix_millis()
            .unwrap_or(0))
    }
}

impl<'a> TryFrom<&'a str> for CPUThread<'a> {
    type Error = Parse;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        let mut scanner = Scanner::new(value);
        let header = scanner
            .take_until("\n")
            .ok_or(Parse::MonitoringHeaderExtraction)?;
        let timestamp = Self::extract_timestamp(header)?;
        scanner.skip_whitespace();
        scanner
            .expect("Thread Info:")
            .map_err(Parse::MonitoringThreadInfoExtraction)?;
        scanner
            .expect("Thread Id :")
            .map_err(Parse::MonitoringThreadIdExtraction)?;
        let tid = scanner
            .take_until(",")
            .ok_or(Parse::MonitoringThreadIdCommaExtraction)?;
        let tid = tid.trim().parse::<u64>()?;

        scanner.skip_whitespace();
        scanner
            .expect("has CPU usage :")
            .map_err(Parse::MonitoringThreadCPUUsageExtraction)?;
        let cpu = scanner
            .take_until_exclusive("\n")
            .ok_or(Parse::MonitoringThreadCPUUsageIncompleteLine)?
            .trim()
            .parse::<f32>()?;

        scanner.skip_whitespace();
        scanner
            .expect("Thread Name:")
            .map_err(Parse::MonitoringThreadNameExtraction)?;
        scanner.skip_whitespace();
        let name = scanner
            .take_until(",")
            .ok_or(Parse::MonitoringThreadNameIncompleteLine)?;
        scanner.skip_whitespace();
        scanner
            .expect("Thread State:")
            .map_err(Parse::MonitoringThreadStateExtraction)?;
        scanner.skip_whitespace();

        let state = scanner
            .take_until("\n")
            .ok_or(Parse::MonitoringThreadStateIncompleteLine)?
            .parse()?;

        let trace = if scanner.peek_until("Is executing native code?").is_some() {
            scanner.take_until("Is executing native code?").unwrap()
        } else {
            scanner.remaining()
        };

        let trace = if trace.trim().is_empty() {
            None
        } else {
            Some(Trace::try_from(trace)?)
        };

        Ok(Self {
            state,
            tid,
            cpu,
            name,
            timestamp,
            trace,
        })
    }
}

#[cfg(test)]
mod test {
    use crate::parser::{
        cpumonitoring::{CPUThread, State},
        stacktrace::{Element, Source},
    };

    #[test]
    fn cpumonitoring_empty_trace() {
        let input = r#"[13:27:55.528]|[07-04-2026]|[CPUMonitoring]|[ERROR]|[150207]| :: 
Thread Info:Thread Id : 620, has CPU usage : 0.095
Thread Name: pool-25-thread-15, Thread State: TIMED_WAITING
"#;

        let thread = CPUThread::try_from(input);
        assert!(
            thread.is_ok(),
            "Error during parsing: {:?}",
            thread.unwrap_err()
        );

        let thread = thread.unwrap();
        assert_eq!(thread.tid, 620u64);
        assert_eq!(thread.cpu, 0.095f32);
        assert_eq!(thread.name, "pool-25-thread-15");
        assert_eq!(thread.state, State::TimedWaiting);
        assert!(thread.trace.is_none());
    }

    #[test]
    fn cpumonitoring_full_trace() {
        let input = r#"[13:28:00.450]|[07-04-2026]|[CPUMonitoring]|[ERROR]|[150228]| :: 
Thread Info:Thread Id : 28, has CPU usage : 2.0939937
Thread Name: Glowroot-Aggregate-Flushing, Thread State: BLOCKED
  at org.glowroot.agent.embedded.util.DataSource.update(DataSource.java:359)
  at org.glowroot.agent.embedded.util.DataSource.update(DataSource.java:338)
  at org.glowroot.agent.embedded.repo.FullQueryTextDao.updateLastCaptureTime(FullQueryTextDao.java:77)
  at org.glowroot.agent.embedded.repo.AggregateDao$1.addToTruncatedQueryTexts(AggregateDao.java:228)
  at org.glowroot.agent.embedded.repo.AggregateDao$1.visitOverallAggregate(AggregateDao.java:207)
  at org.glowroot.agent.impl.AggregateIntervalCollector$AggregateReaderImpl.accept(AggregateIntervalCollector.java:318)
  at org.glowroot.agent.embedded.repo.AggregateDao.store(AggregateDao.java:203)
  at org.glowroot.agent.embedded.init.EmbeddedCollector.collectAggregates(EmbeddedCollector.java:82)
  at org.glowroot.agent.init.CollectorProxy.collectAggregates(CollectorProxy.java:55)
  at org.glowroot.agent.impl.AggregateIntervalCollector.flush(AggregateIntervalCollector.java:215)
  at org.glowroot.agent.impl.TransactionProcessor$AggregateFlushingLoop.run(TransactionProcessor.java:298)
  at java.base@17.0.18/java.util.concurrent.ThreadPoolExecutor.runWorker(Unknown Source)
  at java.base@17.0.18/java.util.concurrent.ThreadPoolExecutor$Worker.run(Unknown Source)
  at java.base@17.0.18/java.lang.Thread.run(Unknown Source)
Is executing native code? : false"#;
        let thread = CPUThread::try_from(input);
        assert!(
            thread.is_ok(),
            "Error during parsing: {:?}",
            thread.unwrap_err()
        );
        let thread = thread.unwrap();
        // NOTE: We check for types to, check whether we have to use types
        assert_eq!(thread.tid, 28u64);
        assert_eq!(thread.cpu, 2.0939937f32);
        assert_eq!(thread.name, "Glowroot-Aggregate-Flushing");
        assert_eq!(thread.state, State::Blocked);
        assert!(thread.trace.is_some(), "Error during parsing stacktrace");
        let trace = thread.trace.unwrap();
        assert_eq!(trace.0.len(), 14);
        assert_eq!(
            trace.0.first(),
            Some(Element::Elem {
                method: "org.glowroot.agent.embedded.util.DataSource.update",
                source: Source::Filename {
                    file: "DataSource.java",
                    line: 359u64
                }
            })
            .as_ref()
        );
        assert_eq!(
            trace.0.last(),
            Some(Element::Elem {
                method: "java.base@17.0.18/java.lang.Thread.run",
                source: Source::UnknownSource
            })
            .as_ref()
        );
    }
}
