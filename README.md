# rjprof

- Java profiler written in Rust, similar to [async-profiler](https://github.com/async-profiler/async-profiler).

## Usage

## Current State

- It "works" for now. Obviously, it's pretty early.

➜  rjprof git:(main) ✗ java -agentpath:$(pwd)/target/release/librjprof.dylib -jar HelloApp.jar
🔗 Agent attached, waiting for VM_INIT...
✅ [VM_INIT] JVM thread count: 4
Hello from Java!
Starting sleeping...

=== Top 20 slowest methods (avg ns) ===
Method                                                               Calls       Avg(ns)     Total(ns)

---------------------------------------------------------------------------------------------------------------------------------------------------------------------------

main([Ljava/lang/String;)V                                               1 10005844125ns 10005844125ns
sleep(J)V                                                                1 10005343500ns 10005343500ns
sleepNanos(J)V                                                           1 10005332542ns 10005332542ns
sleepNanos0(J)V                                                          1 10005135667ns 10005135667ns
checkAndLoadMain(ZILjava/lang/String;)Ljava/lang/Class;                  1    14971333ns    14971333ns
loadMainClass(ILjava/lang/String;)Ljava/lang/Class;                      1    14549250ns    14549250ns
<init>(Ljava/lang/String;)V                                              1     6534542ns     6534542ns
<init>(Ljava/io/File;ZI)V                                                1     6510667ns     6510667ns
getMainClassFromJar(Ljava/util/jar/JarFile;)Ljava/lang/String;           1     4074500ns     4074500ns
<clinit>()V                                                              1     3462417ns     3462417ns
<clinit>()V                                                              1     3436292ns     3436292ns
<init>(Ljava/io/File;ZILjava/lang/Runtime$Version;)V                     2     3431021ns     6862042ns
<init>(Ljava/io/File;I)V                                                 2     3419604ns     6839208ns
<init>(Ljava/io/File;ILjava/nio/charset/Charset;)V                       2     3418292ns     6836584ns
<init>()V                                                                1     3331000ns     3331000ns
<init>()V                                                                1     3330166ns     3330166ns
<init>()V                                                                1     3329291ns     3329291ns
newFileSystem(Ljava/lang/String;)Lsun/nio/fs/UnixFileSystem;             1     3323250ns     3323250ns
newFileSystem(Ljava/lang/String;)Lsun/nio/fs/MacOSXFileSystem;           1     3322291ns     3322291ns
forName(Ljava/lang/String;ZLjava/lang/ClassLoader;)Ljava/lang/Class;     1     3274833ns     3274833ns

