use crate::error::threaddump::Parse;
use crate::threaddump::{Element, Object, Source, StackTrace, ThreadState, Thread};
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
fn test_object_id_hex_invalid_length() {
    let input = "199d244dx";
    let result = Object::hex_to_u64(input);
    assert_eq!(
        result.unwrap_err(),
        Parse::HexLen {
            expected: 8,
            got: 9
        }
    );
}

#[test]
fn test_object_id_hex_invalid_digit() {
    let input = "199d244x";
    let result = Object::hex_to_u64(input);
    assert_eq!(result.unwrap_err(), Parse::HexUnexpectedChar { got: 'x' });
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
fn test_source_filename_no_colon() -> Result<(), Parse> {
    let input = "NioEndpoint.java;10";
    let result = Source::try_from(input);
    assert!(result.is_err(), "Got error: {:?}", result.unwrap_err());
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
            object_id: 1357860604
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
    let input = "Java.lang.Thread.State: NEW";
    let result = ThreadState::try_from(input);
    assert!(result.is_ok(), "Got error: {result:?}");
    assert_eq!(Result::Ok(ThreadState::New), result);
    Ok(())
}

#[test]
fn test_thread_state_terminated() -> Result<(), Parse> {
    let input = "Java.lang.Thread.State: TERMINATED";
    let result = ThreadState::try_from(input);
    assert!(result.is_ok(), "Got error: {result:?}");
    assert_eq!(Result::Ok(ThreadState::Terminated), result);
    Ok(())
}

#[test]
fn test_thread_state_runnable() -> Result<(), Parse> {
    let input = "Java.lang.Thread.State: RUNNABLE";
    let result = ThreadState::try_from(input);
    assert!(result.is_ok(), "Got error: {result:?}");
    assert_eq!(Result::Ok(ThreadState::Runnable), result);
    Ok(())
}

#[test]
fn test_thread_state_waiting() -> Result<(), Parse> {
    let input = "Java.lang.Thread.State: WAITING on java.util.concurrent.locks.AbstractQueuedSynchronizer$ConditionObject@586806fa";
    let result = ThreadState::try_from(input);
    assert!(result.is_ok(), "Got error: {result:?}");
    assert_eq!(result, Result::Ok(ThreadState::Waiting(Object {
        class: "java.util.concurrent.locks.AbstractQueuedSynchronizer$ConditionObject",
        object_id: 1483212538
    })));
    Ok(())
}

#[test]
fn test_thread_state_timed_waiting() -> Result<(), Parse> {
    let input = "Java.lang.Thread.State: TIMED_WAITING on java.util.concurrent.locks.AbstractQueuedSynchronizer$ConditionObject@586806fa";
    let result = ThreadState::try_from(input);
    assert!(result.is_ok(), "Got error: {result:?}");
    assert_eq!(result, Result::Ok(ThreadState::TimedWaiting(Object {
        class: "java.util.concurrent.locks.AbstractQueuedSynchronizer$ConditionObject",
        object_id: 1483212538
    })));
    Ok(())
}

#[test]
fn test_thread_state_blocked() -> Result<(), Parse> {
    let input = "Java.lang.Thread.State: BLOCKED on java.util.concurrent.locks.AbstractQueuedSynchronizer$ConditionObject@586806fa";
    let result = ThreadState::try_from(input);
    assert!(result.is_ok(), "Got error: {result:?}");
    assert_eq!(result, Result::Ok(ThreadState::Blocked(Object {
        class: "java.util.concurrent.locks.AbstractQueuedSynchronizer$ConditionObject",
        object_id: 1483212538
    })));
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
    let input = r#""Thread-10"  Id=60  Java.lang.Thread.State: WAITING on java.util.concurrent.locks.AbstractQueuedSynchronizer$ConditionObject@586806fa
 java.base@17.0.17/jdk.internal.misc.Unsafe.park(Native Method)
 java.base@17.0.17/java.util.concurrent.locks.LockSupport.park(Unknown Source)
 java.base@17.0.17/java.util.concurrent.locks.AbstractQueuedSynchronizer$ConditionNode.block(Unknown Source)
 java.base@17.0.17/java.util.concurrent.ForkJoinPool.unmanagedBlock(Unknown Source)
 java.base@17.0.17/java.util.concurrent.ForkJoinPool.managedBlock(Unknown Source)
 java.base@17.0.17/java.util.concurrent.locks.AbstractQueuedSynchronizer$ConditionObject.await(Unknown Source)
 java.base@17.0.17/java.util.concurrent.LinkedBlockingDeque.takeFirst(Unknown Source)
 java.base@17.0.17/java.util.concurrent.LinkedBlockingDeque.take(Unknown Source)
 java.base@17.0.17/sun.nio.fs.AbstractWatchService.take(Unknown Source)
 app//com.zoho.conf.WatchFile.run(WatchFile.java:39)"#;
    let result = Thread::try_from(input);
    assert!(result.is_ok(), "Got error: {result:?}");
    println!("{result:?}");
    Ok(())
}
