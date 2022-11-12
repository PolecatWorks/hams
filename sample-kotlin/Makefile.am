
CARGO_TARGET =$(if $(RELEASE),target/release,target/debug)
CARGO_BUILD_FLAGS =$(if $(RELEASE),--release,)

$(top_build_prefix)$(CARGO_TARGET)/libhams.dylib:
	cd $(top_srcdir) && make $(CARGO_TARGET)/libhams.dylib

libhams.dylib: $(top_build_prefix)$(CARGO_TARGET)/libhams.dylib
	cp $(top_build_prefix)$(CARGO_TARGET)/libhams.dylib ./


export JAVA_HOME = $(shell /usr/libexec/java_home -v 19.0.1)

target/target/my-maven-sample-app-1.0-SNAPSHOT-jar-with-dependencies.jar:
	@mvn package


clean-local:
	@mvn clean

check-local: libhams.dylib
check-local: target/target/my-maven-sample-app-1.0-SNAPSHOT-jar-with-dependencies.jar
	@java -jar target/my-maven-sample-app-1.0-SNAPSHOT-jar-with-dependencies.jar

# https://medium.com/codex/getting-started-with-kotlin-and-maven-9482a65158a1
init-maven:
	mvn archetype:generate -DgroupId=com.polecatworks.kotlin.samples -DartifactId=my-maven-sample-app -DarchetypeGroupId=org.jetbrains.kotlin -DarchetypeArtifactId=kotlin-archetype-jvm -DarchetypeVersion=1.7.20 -DinteractiveMode=false
