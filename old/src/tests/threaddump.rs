use std::collections::HashMap;
use std::{fs::{self}, path::PathBuf};
use std::process::exit;

use time::PrimitiveDateTime;

use crate::error::threaddump::Parse;
use crate::threaddump::{Element, LockInfo, Object, Source, StackTrace, Thread, ThreadDump, ThreadDumpStreamer, ThreadState};
use crate::util;
#[test]
fn test_object_id_parse() -> Result<(), Parse> {
    let input = "199d244d";
    let result = Object::hex_to_u64(input)?;
    assert_eq!(result, 429728845);
    Ok(())
}

#[test]
fn test_object_id_zero() -> Result<(), Parse> {
    let input = "00000000";
    let result = Object::hex_to_u64(input)?;
    assert_eq!(result, 0);
    Ok(())
}

#[test]
fn test_object_id_overflow() -> Result<(), Parse> {
    let input = "ffffffff";
    let result = Object::hex_to_u64(input)?;
    assert_eq!(result, 4294967295);
    Ok(())
}

#[test]
fn test_object_id_hex_invalid_digit() {
    let input = "199d244x";
    let result = Object::hex_to_u64(input);
    assert_eq!(result.unwrap_err(), Parse::HexUnexpectedInput { got: String::from("199d244x") });
}

#[test]
fn test_source_unknown() -> Result<(), Parse> {
    let input = "Unknown Source";
    let result = Source::try_from(input)?;
    assert_eq!(result, Source::UnknownSource);
    Ok(())
}

#[test]
fn test_source_native() -> Result<(), Parse> {
    let input = "Native Method";
    let result = Source::try_from(input)?;
    assert_eq!(result, Source::NativeMethod);
    Ok(())
}

#[test]
fn test_source_generated() -> Result<(), Parse> {
    let input = "Lambda$10";
    let result = Source::try_from(input)?;
    assert_eq!(result, Source::Generated("Lambda$10"));
    Ok(())
}

#[test]
fn test_source_filename() -> Result<(), Parse> {
    let input = "NioEndpoint.java:10";
    let result = Source::try_from(input)?;
    assert_eq!(
        result,
        Source::Filename {
            file: "NioEndpoint.java",
            line_number: 10
        }
    );
    Ok(())
}

#[test]
fn test_source_filename_invalid_line_number() -> Result<(), Parse> {
    let input = "NioEndpoint.java:10.10";
    let result = Source::try_from(input);
    assert!(result.is_err(), "Got error: {:?}", result.unwrap_err());
    Ok(())
}

#[test]
fn test_element_locked() -> Result<(), Parse> {
    let input = "- locked java.io.BufferedInputStream@50ef4efc";
    let result = Element::try_from(input);
    assert!(result.is_ok(), "Got error: {:?}", result);
    let result = result?;
    assert_eq!(
        result,
        Element::Lock(Object {
            class: "java.io.BufferedInputStream",
            identity: 1357860604
        })
    );
    Ok(())
}

#[test]
fn test_element_frame() -> Result<(), Parse> {
    let input = "com.manageengine.servicedesk.perf.monitoring.WindowsSystemMonitor.getSystemMetrics(WindowsSystemMonitor.java:40)";
    let result = Element::try_from(input);
    assert!(result.is_ok(), "Got error: {:?}", result);
    let result = result?;
    assert_eq!(
        result,
        Element::Elem {
            frame: "com.manageengine.servicedesk.perf.monitoring.WindowsSystemMonitor.getSystemMetrics",
            source: Source::Filename {
                file: "WindowsSystemMonitor.java",
                line_number: 40
            }
        }
    );
    Ok(())
}

#[test]
fn test_element_frame_unknown_source() -> Result<(), Parse> {
    let input = "java.base@17.0.17/java.io.BufferedReader.readLine(Unknown Source)";
    let result = Element::try_from(input);
    assert!(result.is_ok(), "Got error: {result:?}");
    let result = result?;
    assert_eq!(
        result,
        Element::Elem {
            frame: "java.base@17.0.17/java.io.BufferedReader.readLine",
            source: Source::UnknownSource
        }
    );
    Ok(())
}

#[test]
fn test_stacktrace_simple() -> Result<(), Parse> {
    let input = r#"
    java.base@17.0.17/jdk.internal.misc.Unsafe.park(Native Method)
 java.base@17.0.17/java.util.concurrent.locks.LockSupport.park(Unknown Source)
 java.base@17.0.17/java.util.concurrent.locks.AbstractQueuedSynchronizer$ConditionNode.block(Unknown Source)
 java.base@17.0.17/java.util.concurrent.ForkJoinPool.unmanagedBlock(Unknown Source)
 java.base@17.0.17/java.util.concurrent.ForkJoinPool.managedBlock(Unknown Source)
 java.base@17.0.17/java.util.concurrent.locks.AbstractQueuedSynchronizer$ConditionObject.await(Unknown Source)
 java.base@17.0.17/java.util.concurrent.LinkedBlockingDeque.takeFirst(Unknown Source)
 java.base@17.0.17/java.util.concurrent.LinkedBlockingDeque.take(Unknown Source)
 java.base@17.0.17/sun.nio.fs.AbstractWatchService.take(Unknown Source)
 com.zoho.delta.agent.DeltaAgent$AgentConfWatcher.run(DeltaAgent.java:123)
 java.base@17.0.17/java.lang.Thread.run(Unknown Source)"#;

    let result = StackTrace::try_from(input);
    assert!(result.is_ok(), "Got error: {result:?}");
    Ok(())
}

#[test]
fn test_stacktrace_with_locks() -> Result<(), Parse> {
    let input = r#"
 java.base@17.0.17/sun.nio.ch.WEPoll.wait(Native Method)
 java.base@17.0.17/sun.nio.ch.WEPollSelectorImpl.doSelect(Unknown Source)
 java.base@17.0.17/sun.nio.ch.SelectorImpl.lockAndDoSelect(Unknown Source)
 - locked org.glowroot.agent.shaded.io.netty.channel.nio.SelectedSelectionKeySet@63cfa33b
 - locked sun.nio.ch.WEPollSelectorImpl@7dbac81a
 java.base@17.0.17/sun.nio.ch.SelectorImpl.select(Unknown Source)
 org.glowroot.agent.shaded.io.netty.channel.nio.SelectedSelectionKeySetSelector.select(SelectedSelectionKeySetSelector.java:68)
 org.glowroot.agent.shaded.io.netty.channel.nio.NioEventLoop.select(NioEventLoop.java:879)
 org.glowroot.agent.shaded.io.netty.channel.nio.NioEventLoop.run(NioEventLoop.java:526)
 org.glowroot.agent.shaded.io.netty.util.concurrent.SingleThreadEventExecutor$4.run(SingleThreadEventExecutor.java:997)
 org.glowroot.agent.shaded.io.netty.util.internal.ThreadExecutorMap$2.run(ThreadExecutorMap.java:74)
 java.base@17.0.17/java.lang.Thread.run(Unknown Source)"#;

    let result = StackTrace::try_from(input);
    assert!(result.is_ok(), "Got error: {result:?}");
    Ok(())
}

#[test]
fn test_thread_state_new() -> Result<(), Parse> {
    let input = "Java.lang.Thread.State: NEW\n";
    let result = ThreadState::try_from(input);
    assert!(result.is_ok(), "Got error: {result:?}");
    assert_eq!(Result::Ok(ThreadState::New), result);
    Ok(())
}

#[test]
fn test_thread_state_terminated() -> Result<(), Parse> {
    let input = "Java.lang.Thread.State: TERMINATED\n";
    let result = ThreadState::try_from(input);
    assert!(result.is_ok(), "Got error: {result:?}");
    assert_eq!(Result::Ok(ThreadState::Terminated), result);
    Ok(())
}

#[test]
fn test_thread_state_runnable() -> Result<(), Parse> {
    let input = "Java.lang.Thread.State: RUNNABLE\n";
    let result = ThreadState::try_from(input);
    assert!(result.is_ok(), "Got error: {result:?}");
    assert_eq!(Result::Ok(ThreadState::Runnable), result);
    Ok(())
}

#[test]
fn test_thread_state_waiting() -> Result<(), Parse> {
    let input = "Java.lang.Thread.State: WAITING on java.util.concurrent.locks.AbstractQueuedSynchronizer$ConditionObject@586806fa\n";
    let result = ThreadState::try_from(input);
    assert!(result.is_ok(), "Got error: {result:?}");
    assert_eq!(
        result,
        Result::Ok(ThreadState::WaitingOn(Object {
            class: "java.util.concurrent.locks.AbstractQueuedSynchronizer$ConditionObject",
            identity: 1483212538
        }))
    );
    Ok(())
}

#[test]
fn test_thread_state_timed_waiting() -> Result<(), Parse> {
    let input = "Java.lang.Thread.State: TIMED_WAITING on java.util.concurrent.locks.AbstractQueuedSynchronizer$ConditionObject@586806fa\n";
    let result = ThreadState::try_from(input);
    assert!(result.is_ok(), "Got error: {result:?}");
    assert_eq!(
        result,
        Result::Ok(ThreadState::TimedWaitingOn(Object {
            class: "java.util.concurrent.locks.AbstractQueuedSynchronizer$ConditionObject",
            identity: 1483212538
        }))
    );
    Ok(())
}

#[test]
fn test_thread_state_unrecognized() -> Result<(), Parse> {
    let input = "Java.lang.Thread.State: WHAT THE FUCK";
    let result = ThreadState::try_from(input);
    assert!(result.is_err(), "Got error: {result:?}");
    assert_eq!(Result::Err(Parse::UnexpectedThreadState), result);
    Ok(())
}

#[test]
fn test_thread() -> Result<(), Parse> {
    let input = r#"
"Glowroot-Trace-Collector"  Id=26  Java.lang.Thread.State: BLOCKED waiting to lock java.lang.Object@73853f10
 LockName: java.lang.Object@73853f10 Owner Id: 28 Owner Name: Glowroot-Aggregate-Flushing
 org.glowroot.agent.embedded.util.DataSource.update(DataSource.java:359)
 org.glowroot.agent.embedded.repo.TraceDao.store(TraceDao.java:185)
 org.glowroot.agent.embedded.init.EmbeddedCollector.collectTrace(EmbeddedCollector.java:144)
 org.glowroot.agent.init.CollectorProxy.collectTrace(CollectorProxy.java:77)
 org.glowroot.agent.impl.TraceCollector$TraceCollectorLoop.collectCompleted(TraceCollector.java:321)
 org.glowroot.agent.impl.TraceCollector$TraceCollectorLoop.run(TraceCollector.java:296)
 java.base@17.0.17/java.util.concurrent.ThreadPoolExecutor.runWorker(Unknown Source)
 java.base@17.0.17/java.util.concurrent.ThreadPoolExecutor$Worker.run(Unknown Source)
 java.base@17.0.17/java.lang.Thread.run(Unknown Source)
 "#;
    let result = Thread::try_from(input);
    assert!(result.is_ok(), "Got error: {result:?}");
    let result = result.unwrap();
    assert!(result.stacktrace.is_some(), "Got None stacktrace");
    assert_eq!(
        result.state,
        ThreadState::BlockedToLock(Some(LockInfo {
            owner_id: 28,
            owner_name: Some("Glowroot-Aggregate-Flushing"),
            object: Object {
                class: "java.lang.Object",
                identity: 1938112272
            }
        }))
    );
    assert_eq!(result.stacktrace.unwrap().elems.len(), 9);
    Ok(())
}

#[test]
fn test_thread_locked_runnable() -> Result<(), Parse> {
    let input = r#"
"Glowroot-Aggregate-Flushing"  Id=28  Java.lang.Thread.State: RUNNABLE
 java.base@17.0.17/java.io.FileDescriptor.sync(Native Method)
 org.glowroot.agent.embedded.shaded.org.h2.store.fs.FileDisk.force(FilePathDisk.java:410)
 org.glowroot.agent.embedded.shaded.org.h2.store.FileStore.sync(FileStore.java:419)
 org.glowroot.agent.embedded.shaded.org.h2.store.PageStore.writeVariableHeader(PageStore.java:982)
 org.glowroot.agent.embedded.shaded.org.h2.store.PageStore.setLogFirstPage(PageStore.java:976)
 org.glowroot.agent.embedded.shaded.org.h2.store.PageLog.removeUntil(PageLog.java:726)
 org.glowroot.agent.embedded.shaded.org.h2.store.PageStore.checkpoint(PageStore.java:441)
 - locked org.glowroot.agent.embedded.shaded.org.h2.store.PageStore@3c7fccd4
 org.glowroot.agent.embedded.shaded.org.h2.store.PageStore.commit(PageStore.java:1481)
 - locked org.glowroot.agent.embedded.shaded.org.h2.store.PageStore@3c7fccd4
 org.glowroot.agent.embedded.shaded.org.h2.engine.Database.commit(Database.java:1926)
 - locked org.glowroot.agent.embedded.shaded.org.h2.engine.Database@32b8352b
 org.glowroot.agent.embedded.shaded.org.h2.engine.Session.commit(Session.java:494)
 org.glowroot.agent.embedded.shaded.org.h2.command.Command.stop(Command.java:152)
 org.glowroot.agent.embedded.shaded.org.h2.command.Command.executeUpdate(Command.java:284)
 - locked org.glowroot.agent.embedded.shaded.org.h2.engine.Database@32b8352b
 org.glowroot.agent.embedded.shaded.org.h2.jdbc.JdbcPreparedStatement.executeUpdateInternal(JdbcPreparedStatement.java:158)
 - locked org.glowroot.agent.embedded.shaded.org.h2.engine.Session@3687a7a4
 org.glowroot.agent.embedded.shaded.org.h2.jdbc.JdbcPreparedStatement.executeUpdate(JdbcPreparedStatement.java:144)
 org.glowroot.agent.embedded.util.DataSource.update(DataSource.java:366)
 - locked java.lang.Object@73853f10
 org.glowroot.agent.embedded.util.DataSource.update(DataSource.java:338)
 org.glowroot.agent.embedded.repo.FullQueryTextDao.updateLastCaptureTime(FullQueryTextDao.java:77)
 - locked java.lang.Object@315e54d8
 org.glowroot.agent.embedded.repo.AggregateDao$1.addToTruncatedQueryTexts(AggregateDao.java:228)
 org.glowroot.agent.embedded.repo.AggregateDao$1.visitOverallAggregate(AggregateDao.java:207)
 org.glowroot.agent.impl.AggregateIntervalCollector$AggregateReaderImpl.accept(AggregateIntervalCollector.java:318)
 org.glowroot.agent.embedded.repo.AggregateDao.store(AggregateDao.java:203)
 org.glowroot.agent.embedded.init.EmbeddedCollector.collectAggregates(EmbeddedCollector.java:82)
 org.glowroot.agent.init.CollectorProxy.collectAggregates(CollectorProxy.java:55)
 org.glowroot.agent.impl.AggregateIntervalCollector.flush(AggregateIntervalCollector.java:215)
 org.glowroot.agent.impl.TransactionProcessor$AggregateFlushingLoop.run(TransactionProcessor.java:298)
 java.base@17.0.17/java.util.concurrent.ThreadPoolExecutor.runWorker(Unknown Source)
 java.base@17.0.17/java.util.concurrent.ThreadPoolExecutor$Worker.run(Unknown Source)
    "#;

    let result = Thread::try_from(input);
    assert!(result.is_ok(), "Got error: {result:?}");
    println!("{result:?}");
    Ok(())
}

#[test]
fn test_thread_no_stacktrace() -> Result<(), Parse> {
    let input = "\"Glowroot-Aggregate-Flushing\"  Id=28  Java.lang.Thread.State: RUNNABLE\r\n";
    let thread = Thread::try_from(input);
    assert!(thread.is_ok(), "Got Error: {thread:?}");
    let thread = thread?;
    assert_eq!(thread.state, ThreadState::Runnable);
    assert_eq!(thread.thread_name, Some("Glowroot-Aggregate-Flushing"));
    assert_eq!(thread.thread_id, 28);
    assert_eq!(thread.stacktrace, None);
    Ok(())
}

#[test]
fn test_thread_no_stacktrace_waiting() -> Result<(), Parse> {
    let input = r#""Finalizer"  Id=3  Java.lang.Thread.State: WAITING on java.lang.ref.ReferenceQueue$Lock@15c8762
"#;
    let thread = Thread::try_from(input);
    assert!(thread.is_ok(), "Got error: {thread:?}");
    let thread = thread?;
    assert_eq!(
        thread.state,
        ThreadState::WaitingOn(Object {
            class: "java.lang.ref.ReferenceQueue$Lock",
            identity: 22841186
        })
    );
    assert_eq!(thread.thread_name, Some("Finalizer"));
    assert_eq!(thread.thread_id, 3);
    assert_eq!(thread.stacktrace, None);
    Ok(())
}

#[test]
fn test_threaddump_timestamp_parsing() -> Result<(), Parse> {
    let input = "2026-02-25 18:20:16.093";
    let result = PrimitiveDateTime::parse(input, ThreadDump::FORMAT);
    assert!(result.is_ok(), "Got Error: {result:?}");
    Ok(())
}

#[test]
fn test_threaddump_no_stacktraces_continuous() -> Result<(), Parse> {
    let input = r#"
Thread dump : 1 : 2026-02-25 18:20:16.093

"Signal Dispatcher"  Id=4  Java.lang.Thread.State: RUNNABLE

"Attach Listener"  Id=5  Java.lang.Thread.State: RUNNABLE

"Notification Thread"  Id=24  Java.lang.Thread.State: RUNNABLE
    "#;
    let result = ThreadDump::try_from(input)?;
    println!("{result:?}");
    Ok(())
}

#[test]
fn test_threaddump_partial() -> Result<(), Parse> {
    let input = r#"
Thread dump : 1 : 2026-02-25 18:20:16.093

"Reference Handler"  Id=2  Java.lang.Thread.State: RUNNABLE
 java.base@17.0.17/java.lang.ref.Reference.waitForReferencePendingList(Native Method)
 java.base@17.0.17/java.lang.ref.Reference.processPendingReferences(Unknown Source)
 java.base@17.0.17/java.lang.ref.Reference$ReferenceHandler.run(Unknown Source)

"Finalizer"  Id=3  Java.lang.Thread.State: WAITING on java.lang.ref.ReferenceQueue$Lock@15c8762
 java.base@17.0.17/java.lang.Object.wait(Native Method)
 java.base@17.0.17/java.lang.ref.ReferenceQueue.remove(Unknown Source)
 java.base@17.0.17/java.lang.ref.ReferenceQueue.remove(Unknown Source)
 java.base@17.0.17/java.lang.ref.Finalizer$FinalizerThread.run(Unknown Source)

"Signal Dispatcher"  Id=4  Java.lang.Thread.State: RUNNABLE

"Attach Listener"  Id=5  Java.lang.Thread.State: RUNNABLE

"Common-Cleaner"  Id=21  Java.lang.Thread.State: TIMED_WAITING on java.lang.ref.ReferenceQueue$Lock@5631391
 java.base@17.0.17/java.lang.Object.wait(Native Method)
 java.base@17.0.17/java.lang.ref.ReferenceQueue.remove(Unknown Source)
 java.base@17.0.17/jdk.internal.ref.CleanerImpl.run(Unknown Source)
 java.base@17.0.17/java.lang.Thread.run(Unknown Source)
 java.base@17.0.17/jdk.internal.misc.InnocuousThread.run(Unknown Source)

"Glowroot-Stack-Trace-Collector"  Id=23  Java.lang.Thread.State: TIMED_WAITING
 java.base@17.0.17/java.lang.Thread.sleep(Native Method)
 java.base@17.0.17/java.lang.Thread.sleep(Unknown Source)
 java.base@17.0.17/java.util.concurrent.TimeUnit.sleep(Unknown Source)
 org.glowroot.agent.impl.StackTraceCollector$InternalRunnable.run(StackTraceCollector.java:129)
 java.base@17.0.17/java.lang.Thread.run(Unknown Source)

"Notification Thread"  Id=24  Java.lang.Thread.State: RUNNABLE

"Glowroot-Background-0"  Id=25  Java.lang.Thread.State: WAITING on java.util.concurrent.locks.AbstractQueuedSynchronizer$ConditionObject@40618d59
 java.base@17.0.17/jdk.internal.misc.Unsafe.park(Native Method)
 java.base@17.0.17/java.util.concurrent.locks.LockSupport.park(Unknown Source)
 java.base@17.0.17/java.util.concurrent.locks.AbstractQueuedSynchronizer$ConditionNode.block(Unknown Source)
 java.base@17.0.17/java.util.concurrent.ForkJoinPool.unmanagedBlock(Unknown Source)
 java.base@17.0.17/java.util.concurrent.ForkJoinPool.managedBlock(Unknown Source)
 java.base@17.0.17/java.util.concurrent.locks.AbstractQueuedSynchronizer$ConditionObject.await(Unknown Source)
 java.base@17.0.17/java.util.concurrent.ScheduledThreadPoolExecutor$DelayedWorkQueue.take(Unknown Source)
 java.base@17.0.17/java.util.concurrent.ScheduledThreadPoolExecutor$DelayedWorkQueue.take(Unknown Source)
 java.base@17.0.17/java.util.concurrent.ThreadPoolExecutor.getTask(Unknown Source)
 java.base@17.0.17/java.util.concurrent.ThreadPoolExecutor.runWorker(Unknown Source)
 java.base@17.0.17/java.util.concurrent.ThreadPoolExecutor$Worker.run(Unknown Source)
 java.base@17.0.17/java.lang.Thread.run(Unknown Source)

"Glowroot-Trace-Collector"  Id=26  Java.lang.Thread.State: BLOCKED waiting to lock java.lang.Object@73853f10
 LockName: java.lang.Object@73853f10 Owner Id: 28 Owner Name: Glowroot-Aggregate-Flushing
 org.glowroot.agent.embedded.util.DataSource.update(DataSource.java:359)
 org.glowroot.agent.embedded.repo.TraceDao.store(TraceDao.java:185)
 org.glowroot.agent.embedded.init.EmbeddedCollector.collectTrace(EmbeddedCollector.java:144)
 org.glowroot.agent.init.CollectorProxy.collectTrace(CollectorProxy.java:77)
 org.glowroot.agent.impl.TraceCollector$TraceCollectorLoop.collectCompleted(TraceCollector.java:321)
 org.glowroot.agent.impl.TraceCollector$TraceCollectorLoop.run(TraceCollector.java:296)
 java.base@17.0.17/java.util.concurrent.ThreadPoolExecutor.runWorker(Unknown Source)
 java.base@17.0.17/java.util.concurrent.ThreadPoolExecutor$Worker.run(Unknown Source)
 java.base@17.0.17/java.lang.Thread.run(Unknown Source)

"Glowroot-Aggregate-Processing"  Id=27  Java.lang.Thread.State: TIMED_WAITING
 java.base@17.0.17/java.lang.Thread.sleep(Native Method)
 java.base@17.0.17/java.lang.Thread.sleep(Unknown Source)
 java.base@17.0.17/java.util.concurrent.TimeUnit.sleep(Unknown Source)
 org.glowroot.agent.impl.TransactionProcessor$TransactionProcessorLoop.processOne(TransactionProcessor.java:208)
 org.glowroot.agent.impl.TransactionProcessor$TransactionProcessorLoop.run(TransactionProcessor.java:193)
 java.base@17.0.17/java.util.concurrent.ThreadPoolExecutor.runWorker(Unknown Source)
 java.base@17.0.17/java.util.concurrent.ThreadPoolExecutor$Worker.run(Unknown Source)
 java.base@17.0.17/java.lang.Thread.run(Unknown Source)

"Glowroot-Aggregate-Flushing"  Id=28  Java.lang.Thread.State: RUNNABLE
 java.base@17.0.17/java.io.FileDescriptor.sync(Native Method)
 org.glowroot.agent.embedded.shaded.org.h2.store.fs.FileDisk.force(FilePathDisk.java:410)
 org.glowroot.agent.embedded.shaded.org.h2.store.FileStore.sync(FileStore.java:419)
 org.glowroot.agent.embedded.shaded.org.h2.store.PageStore.writeVariableHeader(PageStore.java:982)
 org.glowroot.agent.embedded.shaded.org.h2.store.PageStore.setLogFirstPage(PageStore.java:976)
 org.glowroot.agent.embedded.shaded.org.h2.store.PageLog.removeUntil(PageLog.java:726)
 org.glowroot.agent.embedded.shaded.org.h2.store.PageStore.checkpoint(PageStore.java:441)
 - locked org.glowroot.agent.embedded.shaded.org.h2.store.PageStore@3c7fccd4
 org.glowroot.agent.embedded.shaded.org.h2.store.PageStore.commit(PageStore.java:1481)
 - locked org.glowroot.agent.embedded.shaded.org.h2.store.PageStore@3c7fccd4
 org.glowroot.agent.embedded.shaded.org.h2.engine.Database.commit(Database.java:1926)
 - locked org.glowroot.agent.embedded.shaded.org.h2.engine.Database@32b8352b
 org.glowroot.agent.embedded.shaded.org.h2.engine.Session.commit(Session.java:494)
 org.glowroot.agent.embedded.shaded.org.h2.command.Command.stop(Command.java:152)
 org.glowroot.agent.embedded.shaded.org.h2.command.Command.executeUpdate(Command.java:284)
 - locked org.glowroot.agent.embedded.shaded.org.h2.engine.Database@32b8352b
 org.glowroot.agent.embedded.shaded.org.h2.jdbc.JdbcPreparedStatement.executeUpdateInternal(JdbcPreparedStatement.java:158)
 - locked org.glowroot.agent.embedded.shaded.org.h2.engine.Session@3687a7a4
 org.glowroot.agent.embedded.shaded.org.h2.jdbc.JdbcPreparedStatement.executeUpdate(JdbcPreparedStatement.java:144)
 org.glowroot.agent.embedded.util.DataSource.update(DataSource.java:366)
 - locked java.lang.Object@73853f10
 org.glowroot.agent.embedded.util.DataSource.update(DataSource.java:338)
 org.glowroot.agent.embedded.repo.FullQueryTextDao.updateLastCaptureTime(FullQueryTextDao.java:77)
 - locked java.lang.Object@315e54d8
 org.glowroot.agent.embedded.repo.AggregateDao$1.addToTruncatedQueryTexts(AggregateDao.java:228)
 org.glowroot.agent.embedded.repo.AggregateDao$1.visitOverallAggregate(AggregateDao.java:207)
 org.glowroot.agent.impl.AggregateIntervalCollector$AggregateReaderImpl.accept(AggregateIntervalCollector.java:318)
 org.glowroot.agent.embedded.repo.AggregateDao.store(AggregateDao.java:203)
 org.glowroot.agent.embedded.init.EmbeddedCollector.collectAggregates(EmbeddedCollector.java:82)
 org.glowroot.agent.init.CollectorProxy.collectAggregates(CollectorProxy.java:55)
 org.glowroot.agent.impl.AggregateIntervalCollector.flush(AggregateIntervalCollector.java:215)
 org.glowroot.agent.impl.TransactionProcessor$AggregateFlushingLoop.run(TransactionProcessor.java:298)
 java.base@17.0.17/java.util.concurrent.ThreadPoolExecutor.runWorker(Unknown Source)
 java.base@17.0.17/java.util.concurrent.ThreadPoolExecutor$Worker.run(Unknown Source)
 java.base@17.0.17/java.lang.Thread.run(Unknown Source)

"Log4j2-TF-1-AsyncLogger[AsyncContext@60e53b93]-1"  Id=31  Java.lang.Thread.State: RUNNABLE
 java.base@17.0.17/java.lang.Object.wait(Native Method)
 java.base@17.0.17/java.lang.Object.wait(Unknown Source)
 app//org.apache.logging.log4j.core.async.TimeoutBlockingWaitStrategy.awaitNanos(TimeoutBlockingWaitStrategy.java:108)
 app//org.apache.logging.log4j.core.async.TimeoutBlockingWaitStrategy.waitFor(TimeoutBlockingWaitStrategy.java:67)
 app//com.lmax.disruptor.ProcessingSequenceBarrier.waitFor(ProcessingSequenceBarrier.java:56)
 app//com.lmax.disruptor.BatchEventProcessor.processEvents(BatchEventProcessor.java:159)
 app//com.lmax.disruptor.BatchEventProcessor.run(BatchEventProcessor.java:125)
 java.base@17.0.17/java.lang.Thread.run(Unknown Source)

"Log4j2-TF-48-Scheduled-2"  Id=32  Java.lang.Thread.State: TIMED_WAITING on java.util.concurrent.locks.AbstractQueuedSynchronizer$ConditionObject@204e7216
 java.base@17.0.17/jdk.internal.misc.Unsafe.park(Native Method)
 java.base@17.0.17/java.util.concurrent.locks.LockSupport.parkNanos(Unknown Source)
 java.base@17.0.17/java.util.concurrent.locks.AbstractQueuedSynchronizer$ConditionObject.awaitNanos(Unknown Source)
 java.base@17.0.17/java.util.concurrent.ScheduledThreadPoolExecutor$DelayedWorkQueue.take(Unknown Source)
 java.base@17.0.17/java.util.concurrent.ScheduledThreadPoolExecutor$DelayedWorkQueue.take(Unknown Source)
 java.base@17.0.17/java.util.concurrent.ThreadPoolExecutor.getTask(Unknown Source)
 java.base@17.0.17/java.util.concurrent.ThreadPoolExecutor.runWorker(Unknown Source)
 java.base@17.0.17/java.util.concurrent.ThreadPoolExecutor$Worker.run(Unknown Source)
 java.base@17.0.17/java.lang.Thread.run(Unknown Source)

"Glowroot-Gauge-Flushing"  Id=33  Java.lang.Thread.State: BLOCKED waiting to lock java.lang.Object@73853f10
 LockName: java.lang.Object@73853f10 Owner Id: 28 Owner Name: Glowroot-Aggregate-Flushing
 org.glowroot.agent.embedded.util.DataSource.batchUpdate(DataSource.java:378)
 org.glowroot.agent.embedded.repo.GaugeValueDao.store(GaugeValueDao.java:142)
 org.glowroot.agent.embedded.init.EmbeddedCollector.collectGaugeValues(EmbeddedCollector.java:116)
 org.glowroot.agent.init.CollectorProxy.collectGaugeValues(CollectorProxy.java:66)
 org.glowroot.agent.init.GaugeCollector$GaugeFlushingLoop.run(GaugeCollector.java:372)
 java.base@17.0.17/java.util.concurrent.ThreadPoolExecutor.runWorker(Unknown Source)
 java.base@17.0.17/java.util.concurrent.ThreadPoolExecutor$Worker.run(Unknown Source)
 java.base@17.0.17/java.lang.Thread.run(Unknown Source)

"Glowroot-Gauge-Collection"  Id=34  Java.lang.Thread.State: TIMED_WAITING on java.util.concurrent.locks.AbstractQueuedSynchronizer$ConditionObject@52ad5ae3
 java.base@17.0.17/jdk.internal.misc.Unsafe.park(Native Method)
 java.base@17.0.17/java.util.concurrent.locks.LockSupport.parkNanos(Unknown Source)
 java.base@17.0.17/java.util.concurrent.locks.AbstractQueuedSynchronizer$ConditionObject.awaitNanos(Unknown Source)
 java.base@17.0.17/java.util.concurrent.ScheduledThreadPoolExecutor$DelayedWorkQueue.take(Unknown Source)
 java.base@17.0.17/java.util.concurrent.ScheduledThreadPoolExecutor$DelayedWorkQueue.take(Unknown Source)
 java.base@17.0.17/java.util.concurrent.ThreadPoolExecutor.getTask(Unknown Source)
 java.base@17.0.17/java.util.concurrent.ThreadPoolExecutor.runWorker(Unknown Source)
 java.base@17.0.17/java.util.concurrent.ThreadPoolExecutor$Worker.run(Unknown Source)
 java.base@17.0.17/java.lang.Thread.run(Unknown Source)

"Glowroot-Background-1"  Id=35  Java.lang.Thread.State: TIMED_WAITING on java.util.concurrent.locks.AbstractQueuedSynchronizer$ConditionObject@40618d59
 java.base@17.0.17/jdk.internal.misc.Unsafe.park(Native Method)
 java.base@17.0.17/java.util.concurrent.locks.LockSupport.parkNanos(Unknown Source)
 java.base@17.0.17/java.util.concurrent.locks.AbstractQueuedSynchronizer$ConditionObject.awaitNanos(Unknown Source)
 java.base@17.0.17/java.util.concurrent.ScheduledThreadPoolExecutor$DelayedWorkQueue.take(Unknown Source)
 java.base@17.0.17/java.util.concurrent.ScheduledThreadPoolExecutor$DelayedWorkQueue.take(Unknown Source)
 java.base@17.0.17/java.util.concurrent.ThreadPoolExecutor.getTask(Unknown Source)
 java.base@17.0.17/java.util.concurrent.ThreadPoolExecutor.runWorker(Unknown Source)
 java.base@17.0.17/java.util.concurrent.ThreadPoolExecutor$Worker.run(Unknown Source)
 java.base@17.0.17/java.lang.Thread.run(Unknown Source)

"Glowroot-H2 File Lock Watchdog C:/Program Files/ManageEngine/ServiceDesk/glowroot/data/data.lock.db"  Id=38  Java.lang.Thread.State: TIMED_WAITING
 java.base@17.0.17/java.lang.Thread.sleep(Native Method)
 org.glowroot.agent.embedded.shaded.org.h2.store.FileLock.run(FileLock.java:517)
 java.base@17.0.17/java.lang.Thread.run(Unknown Source)

"Glowroot-H2 Log Writer DATA"  Id=39  Java.lang.Thread.State: BLOCKED waiting to lock org.glowroot.agent.embedded.shaded.org.h2.engine.Database@32b8352b
 LockName: org.glowroot.agent.embedded.shaded.org.h2.engine.Database@32b8352b Owner Id: 28 Owner Name: Glowroot-Aggregate-Flushing
 org.glowroot.agent.embedded.shaded.org.h2.engine.Database.flush(Database.java:1958)
 org.glowroot.agent.embedded.shaded.org.h2.store.WriterThread.run(WriterThread.java:87)
 java.base@17.0.17/java.lang.Thread.run(Unknown Source)

"Wrapper-Control-Event-Monitor"  Id=48  Java.lang.Thread.State: TIMED_WAITING
 java.base@17.0.17/java.lang.Thread.sleep(Native Method)
 app//org.tanukisoftware.wrapper.WrapperManager$3.run(WrapperManager.java:1074)

"Java2D Disposer"  Id=50  Java.lang.Thread.State: WAITING on java.lang.ref.ReferenceQueue$Lock@24002ef1
 java.base@17.0.17/java.lang.Object.wait(Native Method)
 java.base@17.0.17/java.lang.ref.ReferenceQueue.remove(Unknown Source)
 java.base@17.0.17/java.lang.ref.ReferenceQueue.remove(Unknown Source)
 java.desktop@17.0.17/sun.java2d.Disposer.run(Unknown Source)
 java.base@17.0.17/java.lang.Thread.run(Unknown Source)

"AWT-Windows"  Id=52  Java.lang.Thread.State: RUNNABLE
 java.desktop@17.0.17/sun.awt.windows.WToolkit.eventLoop(Native Method)
 java.desktop@17.0.17/sun.awt.windows.WToolkit.run(Unknown Source)
 java.base@17.0.17/java.lang.Thread.run(Unknown Source)

"Wrapper-Connection"  Id=54  Java.lang.Thread.State: RUNNABLE
 java.base@17.0.17/sun.nio.ch.SocketDispatcher.read0(Native Method)
 java.base@17.0.17/sun.nio.ch.SocketDispatcher.read(Unknown Source)
 java.base@17.0.17/sun.nio.ch.NioSocketImpl.tryRead(Unknown Source)
 java.base@17.0.17/sun.nio.ch.NioSocketImpl.implRead(Unknown Source)
 java.base@17.0.17/sun.nio.ch.NioSocketImpl.read(Unknown Source)
 java.base@17.0.17/sun.nio.ch.NioSocketImpl$1.read(Unknown Source)
 java.base@17.0.17/java.net.Socket$SocketInputStream.read(Unknown Source)
 java.base@17.0.17/java.net.Socket$SocketInputStream.read(Unknown Source)
 java.base@17.0.17/java.io.DataInputStream.readUnsignedByte(Unknown Source)
 java.base@17.0.17/java.io.DataInputStream.readByte(Unknown Source)
 app//org.tanukisoftware.wrapper.WrapperManager.handleBackend(WrapperManager.java:5872)
 app//org.tanukisoftware.wrapper.WrapperManager.run(WrapperManager.java:6312)
 java.base@17.0.17/java.lang.Thread.run(Unknown Source)

"DestroyJavaVM"  Id=55  Java.lang.Thread.State: RUNNABLE
"#;
    let dump = ThreadDump::try_from(input);
    assert!(dump.is_ok(), "Got Error: {dump:?}");
    let dump = dump?;
    assert_eq!(dump.snapshot, 1);
    assert_eq!(dump.threads.len(), 23);
    Ok(())
}

#[test]
fn test_threaddump_full() {
    let path = PathBuf::from("test/");
    if !path.is_dir() {
        exit(1);
    }

    let result_map: HashMap<&str, usize> = HashMap::from([
        ("threaddump0_only_1.txt", 347),
        ("threaddump0_only_2.txt", 324),
    ]);

    for entry in path.read_dir().unwrap() {
        let path = entry.unwrap().path();
        let filename = path.file_name().unwrap().to_string_lossy().to_string();
        if !filename.starts_with("threaddump0_only") {
            continue;
        }
        println!("{filename:?}");
        let input = fs::read_to_string(path).unwrap();
        let dump = ThreadDump::try_from(input.as_str());
        assert!(dump.is_ok(), "Got Error: {dump:?}");
        let dump = dump.unwrap();
        let expected_thread_count = result_map.get(filename.as_str()).unwrap();
        assert_eq!(dump.threads.len(), *expected_thread_count);
    }
}

#[test]
fn test_threaddump_extraction_singular() {
    let input = util::map_file("test/threaddump0_extraction.txt").unwrap();
    let mut iter = ThreadDumpStreamer(&input);
    let result = iter.next();
    assert!(result.is_some(), "Could not extract dump");
    let result = result.unwrap();
    let dump = ThreadDump::try_from(result);
    assert!(dump.is_ok(), "Got Error during parsing dump: {dump:?}");
    let dump = dump.unwrap();
    assert_eq!(dump.threads.len(), 2);
    assert_eq!(dump.snapshot, 1);
}

#[test]
fn test_threaddump_realworld_parsing() {
    let input = util::map_file("test/threaddump0.txt").unwrap();
    let mut count = 0;
    for chunk in ThreadDumpStreamer(&input) {
        let dump = ThreadDump::try_from(chunk);
        assert!(dump.is_ok(), "Got Error during parsing dump: {dump:?}");
        let dump = dump.unwrap();
        count += 1;
        println!("Dump Len: {:?}, Dump Snapshot: {:?}, Dump Triggered Unix MS: {:?}", dump.threads.len(), dump.snapshot, dump.triggered_unix_ms);
    }
    assert_eq!(count, 6);
}

// ============================================================
// Object::try_from — direct parse via @ separator
// ============================================================

#[test]
fn test_object_try_from_basic() -> Result<(), Parse> {
    let input = "java.lang.Object@73853f10";
    let result = Object::try_from(input)?;
    assert_eq!(result.class, "java.lang.Object");
    assert_eq!(result.identity, 0x73853f10);
    Ok(())
}

#[test]
fn test_object_try_from_inner_class() -> Result<(), Parse> {
    let input = "java.util.concurrent.locks.AbstractQueuedSynchronizer$ConditionObject@586806fa";
    let result = Object::try_from(input)?;
    assert_eq!(
        result.class,
        "java.util.concurrent.locks.AbstractQueuedSynchronizer$ConditionObject"
    );
    assert_eq!(result.identity, 0x586806fa);
    Ok(())
}

#[test]
fn test_object_try_from_missing_at() {
    let input = "java.lang.Object-73853f10";
    let result = Object::try_from(input);
    assert_eq!(result.unwrap_err(), Parse::MissingCommat);
}

#[test]
fn test_object_try_from_leading_whitespace() -> Result<(), Parse> {
    let input = "  java.lang.Object@00000001";
    let result = Object::try_from(input)?;
    assert_eq!(result.class, "java.lang.Object");
    assert_eq!(result.identity, 1);
    Ok(())
}

// ============================================================
// LockInfo::try_from — direct parse
// ============================================================

#[test]
fn test_lockinfo_basic() -> Result<(), Parse> {
    let input = "LockName: java.lang.Object@73853f10 Owner Id: 28 Owner Name: Glowroot-Aggregate-Flushing";
    let result = LockInfo::try_from(input)?;
    assert_eq!(result.owner_id, 28);
    assert_eq!(result.owner_name, Some("Glowroot-Aggregate-Flushing"));
    assert_eq!(result.object.class, "java.lang.Object");
    assert_eq!(result.object.identity, 0x73853f10);
    Ok(())
}

#[test]
fn test_lockinfo_empty_owner_name() -> Result<(), Parse> {
    // Owner Name is present but blank after trimming
    let input = "LockName: java.lang.Object@00000001 Owner Id: 5 Owner Name: ";
    let result = LockInfo::try_from(input)?;
    assert_eq!(result.owner_id, 5);
    assert_eq!(result.owner_name, None);
    Ok(())
}

#[test]
fn test_lockinfo_missing_lockname_prefix() {
    let input = "NotALock: java.lang.Object@00000001 Owner Id: 5 Owner Name: X";
    let result = LockInfo::try_from(input);
    assert_eq!(result.unwrap_err(), Parse::ExpectedLockName);
}

#[test]
fn test_lockinfo_missing_owner_id_prefix() {
    let input = "LockName: java.lang.Object@00000001 BadOwner 5 Owner Name: X";
    let result = LockInfo::try_from(input);
    assert_eq!(result.unwrap_err(), Parse::ExpectedOwnerId);
}

#[test]
fn test_lockinfo_missing_owner_name_prefix() {
    let input = "LockName: java.lang.Object@00000001 Owner Id: 5 BadName X";
    let result = LockInfo::try_from(input);
    assert_eq!(result.unwrap_err(), Parse::ExpectedOwnerName);
}

// ============================================================
// ThreadState — plain WAITING / TIMED_WAITING (no object)
// ============================================================

#[test]
fn test_thread_state_plain_waiting() -> Result<(), Parse> {
    let input = "Java.lang.Thread.State: WAITING\n";
    let result = ThreadState::try_from(input)?;
    assert_eq!(result, ThreadState::Waiting);
    Ok(())
}

#[test]
fn test_thread_state_plain_timed_waiting() -> Result<(), Parse> {
    let input = "Java.lang.Thread.State: TIMED_WAITING\n";
    let result = ThreadState::try_from(input)?;
    assert_eq!(result, ThreadState::TimedWaiting);
    Ok(())
}

#[test]
fn test_thread_state_blocked_no_lock_info() -> Result<(), Parse> {
    // BLOCKED without a following LockName line — BlockedToLock(None)
    let input = "Java.lang.Thread.State: BLOCKED waiting to lock java.lang.Object@73853f10\n";
    let result = ThreadState::try_from(input)?;
    assert!(matches!(result, ThreadState::BlockedToLock(None)));
    Ok(())
}

#[test]
fn test_thread_state_missing_preamble() {
    let input = "Thread.State: RUNNABLE\n";
    let result = ThreadState::try_from(input);
    assert_eq!(result.unwrap_err(), Parse::ExpectedPreamble);
}

// ============================================================
// Thread — additional edge cases
// ============================================================

#[test]
fn test_thread_empty_name_becomes_none() -> Result<(), Parse> {
    // An empty quoted name should become thread_name = None
    let input = "\"\"  Id=99  Java.lang.Thread.State: RUNNABLE\r\n";
    let thread = Thread::try_from(input)?;
    assert_eq!(thread.thread_name, None);
    assert_eq!(thread.thread_id, 99);
    assert_eq!(thread.state, ThreadState::Runnable);
    Ok(())
}

#[test]
fn test_thread_waiting_to_lock_via_lockname_line() -> Result<(), Parse> {
    // A WAITING thread that is actually waiting to acquire a lock (has LockName line)
    let input = r#"
"some-thread"  Id=10  Java.lang.Thread.State: WAITING on java.lang.Object@73853f10
 LockName: java.lang.Object@73853f10 Owner Id: 28 Owner Name: owner-thread
"#;
    let thread = Thread::try_from(input)?;
    assert!(
        matches!(thread.state, ThreadState::WaitingToLock(_)),
        "expected WaitingToLock, got {:?}",
        thread.state
    );
    if let ThreadState::WaitingToLock(lock) = &thread.state {
        assert_eq!(lock.owner_id, 28);
        assert_eq!(lock.owner_name, Some("owner-thread"));
    }
    Ok(())
}

#[test]
fn test_thread_plain_waiting_no_stacktrace() -> Result<(), Parse> {
    let input = "\"idle-thread\"  Id=7  Java.lang.Thread.State: WAITING\n";
    let thread = Thread::try_from(input)?;
    assert_eq!(thread.state, ThreadState::Waiting);
    assert_eq!(thread.stacktrace, None);
    Ok(())
}

#[test]
fn test_thread_plain_timed_waiting_no_stacktrace() -> Result<(), Parse> {
    let input = "\"timer-thread\"  Id=8  Java.lang.Thread.State: TIMED_WAITING\n";
    let thread = Thread::try_from(input)?;
    assert_eq!(thread.state, ThreadState::TimedWaiting);
    assert_eq!(thread.stacktrace, None);
    Ok(())
}

// ============================================================
// ThreadDump — header parsing errors
// ============================================================

#[test]
fn test_threaddump_invalid_header_no_thread_dump_prefix() {
    let input = "Not a thread dump\n\n";
    let result = ThreadDump::try_from(input);
    assert_eq!(result.unwrap_err(), Parse::ThreadDumpExtraction);
}

#[test]
fn test_threaddump_invalid_header_bad_snapshot() {
    let input = "Thread dump : not_a_number : 2026-02-25 18:20:16.093\n\n";
    let result = ThreadDump::try_from(input);
    assert_eq!(result.unwrap_err(), Parse::DumpSnapshot);
}

#[test]
fn test_threaddump_invalid_header_bad_timestamp() {
    let input = "Thread dump : 1 : NOT-A-TIMESTAMP\n\n";
    let result = ThreadDump::try_from(input);
    assert!(result.is_err());
    // Should be a timestamp parse error wrapped in SnapshotTimestampParsing
    assert!(matches!(result.unwrap_err(), Parse::SnapshotTimestampParsing(_)));
}

#[test]
fn test_threaddump_no_newline() {
    let result = ThreadDump::try_from("no newline here");
    assert_eq!(result.unwrap_err(), Parse::ExpectedNewline);
}

#[test]
fn test_threaddump_empty_threads_section() -> Result<(), Parse> {
    let input = "Thread dump : 2 : 2026-03-01 10:00:00.000\n\n";
    let dump = ThreadDump::try_from(input)?;
    assert_eq!(dump.snapshot, 2);
    assert!(dump.threads.is_empty());
    Ok(())
}

// ============================================================
// ThreadDumpStreamer — edge cases
// ============================================================

#[test]
fn test_threaddump_streamer_empty_input() {
    let input = b"";
    let results: Vec<_> = ThreadDumpStreamer(input).collect();
    assert!(results.is_empty());
}

#[test]
fn test_threaddump_streamer_whitespace_only() {
    let input = b"   \n\n\t\n   ";
    let results: Vec<_> = ThreadDumpStreamer(input).collect();
    assert!(results.is_empty());
}

// ============================================================
// StackTrace (threaddump) — error paths
// ============================================================

#[test]
fn test_threaddump_stacktrace_empty_input() -> Result<(), Parse> {
    // Empty string → no lines → empty element list
    let result = StackTrace::try_from("")?;
    assert!(result.elems.is_empty());
    Ok(())
}

#[test]
fn test_threaddump_stacktrace_only_locks() -> Result<(), Parse> {
    let input = "- locked java.io.BufferedInputStream@50ef4efc\n- locked java.lang.Object@73853f10";
    let result = StackTrace::try_from(input)?;
    assert_eq!(result.elems.len(), 2);
    assert!(matches!(result.elems[0], Element::Lock(_)));
    assert!(matches!(result.elems[1], Element::Lock(_)));
    Ok(())
}

#[test]
fn test_threaddump_stacktrace_mixed_frames_and_locks() -> Result<(), Parse> {
    let input = "org.glowroot.agent.embedded.util.DataSource.update(DataSource.java:366)\n- locked java.lang.Object@73853f10\norg.glowroot.agent.embedded.util.DataSource.update(DataSource.java:338)";
    let result = StackTrace::try_from(input)?;
    assert_eq!(result.elems.len(), 3);
    assert!(matches!(result.elems[0], Element::Elem { .. }));
    assert!(matches!(result.elems[1], Element::Lock(_)));
    assert!(matches!(result.elems[2], Element::Elem { .. }));
    Ok(())
}
