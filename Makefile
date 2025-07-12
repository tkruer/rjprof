# Makefile for rjprof

# build java TestSuite
test:
	javac -d target/classes -cp target/classes TestSuite/*.java
	java -cp target/classes:target/test-classes TestSuite.TestSuite

# build jar
jar:
	mkdir -p target/classes
	javac -d target/classes -cp target/classes src/com/example/Main.java
	jar cfm target/rjprof.jar manifest.txt -C target/classes .

rust-build:	
	RUSTFLAGS="-Awarnings" cargo build --release
	cp target/release/rjprof target/rjprof
