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
main([Ljava/lang/String;)V — calls=1 avg=10006132417ns total=10006132417ns
sleep(J)V — calls=1 avg=10005724500ns total=10005724500ns
sleepNanos(J)V — calls=1 avg=10005718416ns total=10005718416ns
sleepNanos0(J)V — calls=1 avg=10005167625ns total=10005167625ns
checkAndLoadMain(ZILjava/lang/String;)Ljava/lang/Class; — calls=1 avg=15080459ns total=15080459ns
loadMainClass(ILjava/lang/String;)Ljava/lang/Class; — calls=1 avg=14532417ns total=14532417ns
<init>(Ljava/lang/String;)V — calls=1 avg=6974167ns total=6974167ns
<init>(Ljava/io/File;ZI)V — calls=1 avg=6954291ns total=6954291ns
getMainClassFromJar(Ljava/util/jar/JarFile;)Ljava/lang/String; — calls=1 avg=4167500ns total=4167500ns
<clinit>()V — calls=1 avg=4055042ns total=4055042ns
<clinit>()V — calls=1 avg=4034875ns total=4034875ns
<init>()V — calls=1 avg=3738958ns total=3738958ns
<init>()V — calls=1 avg=3738208ns total=3738208ns
<init>()V — calls=1 avg=3737416ns total=3737416ns
newFileSystem(Ljava/lang/String;)Lsun/nio/fs/UnixFileSystem; — calls=1 avg=3732791ns total=3732791ns
newFileSystem(Ljava/lang/String;)Lsun/nio/fs/MacOSXFileSystem; — calls=1 avg=3732000ns total=3732000ns
<init>(Lsun/nio/fs/UnixFileSystemProvider;Ljava/lang/String;)V — calls=1 avg=3678958ns total=3678958ns
<init>(Lsun/nio/fs/UnixFileSystemProvider;Ljava/lang/String;)V — calls=1 avg=3678042ns total=3678042ns
<init>(Lsun/nio/fs/UnixFileSystemProvider;Ljava/lang/String;)V — calls=1 avg=3677250ns total=3677250ns
<init>(Ljava/io/File;ZILjava/lang/Runtime$Version;)V — calls=2 avg=3622854ns total=7245708ns

