
CARGO_TARGET =$(if $(RELEASE),target/release,target/debug)
CARGO_BUILD_FLAGS =$(if $(RELEASE),--release,)

JEXTRACT_EXEC = jextract-19/bin/jextract

java_ffi_log2 = src/main/java/com/polecatworks/ffi_log2/ffi_log2_h.class
java_hams = src/main/java/com/polecatworks/hams/hams.class


$(JEXTRACT_EXEC):
	curl https://download.java.net/java/early_access/jextract/2/openjdk-19-jextract+2-3_macos-x64_bin.tar.gz | gunzip | tar xv
	sudo xattr -r -d com.apple.quarantine jextract-19
	$(JEXTRACT_EXEC) --version


$(top_build_prefix)$(CARGO_TARGET)/libhams.dylib:
	cd $(top_srcdir) && make $(CARGO_TARGET)/libhams.dylib

libhams.dylib: $(top_build_prefix)$(CARGO_TARGET)/libhams.dylib
	cp $(top_build_prefix)$(CARGO_TARGET)/libhams.dylib ./


export JAVA_HOME = $(shell /usr/libexec/java_home -v 19.0.1)
# export JAVA_HOME = $(shell /usr/libexec/java_home -v 17.0.5)

target/target/my-maven-sample-app-1.0-SNAPSHOT-jar-with-dependencies.jar: $(java_ffi_log2) $(java_hams)
	@mvn package

$(java_ffi_log2): $(top_build_prefix)ffi-log2/target/ffi-log2.h
$(java_ffi_log2): $(JEXTRACT_EXEC)
	$(JEXTRACT_EXEC) --source --output src/main/java -t com.polecatworks.ffi_log2 $(top_build_prefix)ffi-log2/target/ffi-log2.h

$(java_hams): $(top_build_prefix)hams/target/hams.h
$(java_hams): $(JEXTRACT_EXEC)
	$(JEXTRACT_EXEC) --source -I $(top_build_prefix)ffi-log2/target -lhams --output src/main/java -t com.polecatworks.hams $(top_build_prefix)hams/target/hams.h


clean-local:
	@mvn clean
	rm -rf src/main/java/polecatworks/ffi_log2
	rm -rf src/main/java/polecatworks/hams

check-local: libhams.dylib
check-local: target/target/my-maven-sample-app-1.0-SNAPSHOT-jar-with-dependencies.jar
	java --enable-native-access=ALL-UNNAMED --enable-preview -jar target/my-maven-sample-app-1.0-SNAPSHOT-jar-with-dependencies.jar
#java --enable-native-access=ALL-UNNAMED  --add-modules jdk.incubator.foreign -jar target/my-maven-sample-app-1.0-SNAPSHOT-jar-with-dependencies.jar

# https://medium.com/codex/getting-started-with-kotlin-and-maven-9482a65158a1
init-maven:
	mvn archetype:generate -DgroupId=com.polecatworks.kotlin.samples -DartifactId=my-maven-sample-app -DarchetypeGroupId=org.jetbrains.kotlin -DarchetypeArtifactId=kotlin-archetype-jvm -DarchetypeVersion=1.7.20 -DinteractiveMode=false
