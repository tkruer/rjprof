# TODO:

- Research Rust JVM integration (JNI/JVMTI via FFI)
- Simple “Hello World” Rust <-> JVM FFI test
- Build minimal JVMTI binding in Rust (use jvmti-rs or custom bindings)
- Write Rust function to list JVM threads or attach to JVM

https://psy-lob-saw.blogspot.com/2016/02/why-most-sampling-java-profilers-are.html


## 1. Add percentages of overall time

Show each method’s share of the total sampled time. 

┌────────────────────────────┬───────┬────────────────┬────────────────┬────────────┐
│ Method …                   │ Calls │ Avg(ns)        │ Total(ns)      │ % of total │
├────────────────────────────┼───────┼────────────────┼────────────────┼────────────┤
│ main([Ljava/lang/String;)V │ 1     │ 10 005 844 125 │ 10 005 844 125 │ 88.2 %     │
├────────────────────────────┼───────┼────────────────┼────────────────┼────────────┤
│ sleep(J)V                  │ 1     │ 10 005 343 500 │ 10 005 343 500 │ 88.1 %     │
├────────────────────────────┼───────┼────────────────┼────────────────┼────────────┤
│ …                          │ …     │ … ns           │ … ns           │ … %        │
└────────────────────────────┴───────┴────────────────┴────────────────┴────────────┘

This immediately tells you how dominant your hot spots are.


## 2. Human‑friendly time units (µs / ms)

Nanoseconds are precise, but for long‑running methods “10 005 844 125 ns” is easier to read if you express it as “10 005 ms” or “10 s”. We can auto‑scale to the largest
unit that keeps the numbers concise:

┌───────────────────────┬──────────────────┬────────────┐
│ Method …              │ Avg(ns)          │ Avg(human) │
├───────────────────────┼──────────────────┼────────────┤
│ main(...)             │ 10 005 844 125ns │ 10 005 ms  │
├───────────────────────┼──────────────────┼────────────┤
│ checkAndLoadMain(...) │ 14 971 333ns     │ 14.97 ms   │
└───────────────────────┴──────────────────┴────────────┘

## 3. Column‑sortable / custom sort order

Right now we sort by average (total/count). It’d be nice to accept a CLI flag to sort by:

    * total time
    * call count
    * name (alphabetical)

 -sort total or -sort calls.


## 4. Threshold filtering

Allow a “hide anything below X ns / below Y % of total” so small noise methods don’t clutter the top‑N:

    rjprof --min-total 1ms   # only show methods whose total >= 1 ms
    rjprof --min-pct 0.1     # only show methods ≥ 0.1% of total


## 5. More distribution stats (p50, p90, p99)

Instead of just count/avg/total, keep a histogram of exit durations so you can print percentiles:

┌──────────────────┬───────┬────────────────┬────────────────┬────────────────┬────────────────┐
│ Method …         │ Calls │ Avg(ns)        │ p50(ns)        │ p90(ns)        │ p99(ns)        │
├──────────────────┼───────┼────────────────┼────────────────┼────────────────┼────────────────┤
│ sleep(J)V        │ 1     │ 10 005 343 500 │ 10 005 343 500 │ 10 005 343 500 │ 10 005 343 500 │
├──────────────────┼───────┼────────────────┼────────────────┼────────────────┼────────────────┤
│ checkAndLoadMain │ 100   │ 150 804 59     │ 14,800,000     │ 15,080,000     │ 15,080,459     │
└──────────────────┴───────┴────────────────┴────────────────┴────────────────┴────────────────┘


## 6. Package‑/class‑level aggregation

Often you care about hot packages rather than individual methods. We could group by the class prefix (e.g. everything in sun.nio.fs) and fold them:

┌───────────────────────────────────┬───────┬────────────┬───────────────┐
│ Package / Class                   │ Calls │ Avg(ns)    │ Total(ns)     │
├───────────────────────────────────┼───────┼────────────┼───────────────┤
│ sun.nio.fs.UnixFileSystemProvider │ 300   │ 3,677,000  │ 1,103,100,000 │
├───────────────────────────────────┼───────┼────────────┼───────────────┤
│ com.myapp.service.UserService     │ 42    │ 12,000,000 │ 504,000,000   │
├───────────────────────────────────┼───────┼────────────┼───────────────┤
│ …                                 │ …     │ …          │ …             │
└───────────────────────────────────┴───────┴────────────┴───────────────┘

## 7. Flame‑graph / call‑tree output

Instrument callers/callees to build a call tree or even emit a folded-stack text file that you can pipe into inferno/flamegraph.pl for a visual flame‑graph:

    com.myapp.Main.main;com.myapp.Foo.doWork 30
    com.myapp.Main.main;java.lang.Thread.sleep 100
    …

## 8. JSON/CSV export for post‑processing

Add an “export” mode so you can pipe data into your own scripts:

    rjprof --json > profile.json
    rjprof --csv  > profile.csv


## 9. Colorize critical hot spots

Use ANSI color (e.g. red for methods > 20% of total, yellow for > 5%, green for “small”) to draw the eye:

    main(...)         1   10 s    ← in red
    checkAndLoadMain 100   15 ms   ← in yellow
    …


## 10. Live / continuous profiling mode

Right now the agent prints only on VM death. You could add a “snapshot” command (e.g. send SIGUSR1 to the process) or have it listen on a socket/HTTP endpoint so you can
grab intermediate snapshots without stopping the JVM.

---------------------------------------------------------------------------------------------------------------------------------------------------------------------------

## 1. Call‑stack context for hot methods

Instead of just recording per‑method counts, capture the calling stack at method‑entry (or exit).  That lets you build a full call graph or dendrogram of “who calls whom”:

    com.myapp.Main.main → com.myapp.Service.doWork → com.myapp.DAO.load
    com.myapp.Main.main → java.lang.Thread.sleep
    …

You can then print a tree or generate a folded‑stack flame‑graph in the usual “caller;callee;… count” form for inferno/flamegraph.pl.


## 2. Line‑level tracing / instrumentation

With JVMTI you can request line‑number events (SetEventNotificationMode – LINE_NUMBER), so you can break down hot spots down to the source‑line rather than just the method:

    MyClass.java:42    calls=150 avg=12000ns total=1_800_000ns
    MyClass.java:47    calls=150 avg=8000ns  total=1_200_000ns

That can point you directly at the “inner loop” source line.


## 3. Exception‑throw profiling

Hook the Exception event to count how many exceptions each method throws (and at what cost), or even capture the stack trace for each thrown exception.  In a lot of
codebases, excessive exceptions (esp. in hot code) are a huge performance sink.


## 4. Monitor contention / lock‑wait tracing

Enable JVMTI’s MonitorContendedEnter / MonitorContendedEntered callbacks and track where threads spend time waiting on synchronized blocks.  You can report:

    java.lang.Object.wait()                calls=200  total_wait=5ms
    com.myapp.CriticalSection.doWork()     calls=50   avg_wait=80µs


## 5. Thread‑state timeline

Record VM thread‑state changes (ThreadStart, ThreadEnd, ThreadStateChange) so you can show, for each thread, a timeline of RUNNABLE ↔ BLOCKED ↔ WAITING.  That can let you
 see, “hey, thread‑5 spent 80 % of its life BLOCKED.”


## 6. GC pause / heap‑usage annotations

JVMTI can notify you of GC start/end.  You could annotate your profiling timeline with GC events (“GC #12 started at 1.234 s, paused 5 ms, reclaimed 20 MB”).
Alternatively, query GetHeapUsage (JVMTI extension) at VM_DEATH to dump final heap stats.


## 7. Allocation profiling

Track object allocations via the ObjectAlloc event (JVMTI).  You can show “top allocators” – methods or classes responsible for the most allocations or bytes allocated.
E.g.:

    com.myapp.Foo.bar() — allocs=5000   bytes=12 MB
    java.lang.String.<init> — allocs=50 000 bytes=6 MB


## 8. JIT / compilation events

Hook CompiledMethodLoad and CompiledMethodUnload to know when methods get JIT‑compiled or deoptimized.  You can then correlate hot spots with JIT activity (e.g. “this
method was never JIT‑compiled” or “inlined 5 methods here”).


## 9. Native‑call tracing

If your app calls JNI or native libraries, you can catch NativeMethodBind and track the time spent inside native calls vs. Java calls.  Or simply tag any method whose name
starts with something native as “native” and summarize.


## 10. Custom user‑markers

Expose an API that lets user code inject markers in the trace—e.g. annotate sections in your own code with a macro or call that logs “BEGIN PHASE X” / “END PHASE X.”  Your
agent can pick up those markers (by watching System.out, or via a small JNI trampoline) and segment the profiling by phase.

## 11. Contextual metadata (thread‑local tags)

Allow the Java app to push a thread‑local “context tag” (e.g. current user ID, request ID, module name) into the JVM TI environment so that every sample is emitted
alongside that tag.  That way you can break down hot methods per request or per tenant.


## 12. Network / I/O event correlation

If you care about I/O as well as CPU, you can hook ClassPrepare → find java/net/Socket or java/io/FileInputStream, and insert entry/exit callbacks around read/write calls.
 Then you can trivially add a “I/O time” column in your report.


## 13. Live “snapshots” on signal or socket

Rather than waiting for VM death, let the agent listen on a socket or catch a signal (SIGUSR1).  When you hit that, it dumps an intermediate snapshot of the current stats.
 That way you can profile long‑running servers without shutting them down.


## 14. Integration with Java Flight Recorder

Provide an option to offload your collected data into a JFR stream (or merge JFR events with your own stats) so you can view everything in jfr GUI or analysis tools.


## 15. Dynamic filters / black‑boxing

Support dynamic exclusion of certain packages or classes from profiling (e.g. -exclude java.*) so you can scope down to your own application code.


### Wrapping up


* **For latency spikes**: line‑level tracing + call stacks + GC pauses.
* **For lock contention**: monitor‑wait tracing + thread‑state timeline.
* **For memory leaks**: allocation profiling + heap‑usage.
* **For CPU hot spots**: call‑stack sampling + hot‑method % breakdown.

