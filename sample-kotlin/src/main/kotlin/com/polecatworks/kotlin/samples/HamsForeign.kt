
package com.polecatworks.kotlin.samples

import java.lang.foreign.*
import java.lang.invoke.MethodHandles
import java.lang.invoke.MethodType
import java.nio.ByteOrder
import com.polecatworks.hams.hams_h
import com.polecatworks.hams.LogParam
import com.polecatworks.hams.`hello_callback$my_cb`
import com.polecatworks.hams.ExternCRecord
import com.polecatworks.hams.RustString
import com.polecatworks.hams.RustStr


// https://blog.arkey.fr/2021/09/04/a-practical-look-at-jep-412-in-jdk17-with-libsodium/
// https://docs.oracle.com/en/java/javase/17/docs/api/jdk.incubator.foreign/jdk/incubator/foreign/package-summary.html
// https://stackoverflow.com/questions/69321128/how-to-call-a-c-function-from-java-17-using-jep-412-foreign-function-memory-a

// internal object IntComparator {
//     @JvmStatic
//     fun intCompare(addr1: MemoryAddress, addr2: MemoryAddress): Int {
//         println("I am going to compare two addresses")
//         return MemoryAccess.getIntAtOffset(MemorySegment.globalNativeSegment(), addr1.toRawLongValue()) -
//                 MemoryAccess.getIntAtOffset(MemorySegment.globalNativeSegment(), addr2.toRawLongValue())
//     }
// }
internal object IntComparator {
    @JvmStatic
    fun intCompare(addr1: MemoryAddress, addr2: MemoryAddress): Int {
        return addr1.get(ValueLayout.JAVA_INT, 0) - addr2.get(ValueLayout.JAVA_INT, 0);
    }
}

internal object HelloCallback {
  @JvmStatic
  fun helloCallback() {
    println("I am the Kotlin callback");
    println("I have been run")
  }
}

class HamsForeign constructor() {



    init {
        println("i am creating HamsForeign");
        System.loadLibrary("hams")
        var session = MemorySession.openImplicit()

        if (false) {
          var linker = Linker.nativeLinker()
          var loaderLookup = SymbolLookup.loaderLookup()




          var hello_world = linker.downcallHandle(
            loaderLookup.lookup("hello_world").get(),
            FunctionDescriptor.ofVoid()
          )

          hello_world.invoke()
          println("JUST did a handmade hello_world")


          var hello_node = linker.downcallHandle(
              loaderLookup.lookup("hello_node").get(),
              FunctionDescriptor.of(ValueLayout.JAVA_LONG)
          )

          var mynode = hello_node.invoke() as Long
          println("hello_node replied with ${mynode}")


          val intCompareDescriptor = FunctionDescriptor.of(ValueLayout.JAVA_INT, ValueLayout.ADDRESS, ValueLayout.ADDRESS)
          val intCompareHandle = MethodHandles.lookup().findStatic(
            IntComparator::class.java,
            "intCompare",
            Linker.upcallType(intCompareDescriptor)
          )

          // --------------------------------------------------------------
          // Handcraft  to create a Java Call back function to log

          val helloCallbackDescriptor = FunctionDescriptor.ofVoid()
          val helloCallbackHandle = MethodHandles.lookup().findStatic(
            HelloCallback::class.java,
            "helloCallback",
            Linker.upcallType(helloCallbackDescriptor)
          )


          val helloCallbackNativeSymbol = linker.upcallStub(
            helloCallbackHandle, helloCallbackDescriptor, session
          )

          println("About to make callback [${helloCallbackNativeSymbol} ]")
          hams_h.hello_callback(helloCallbackNativeSymbol)

          // --------------------------------------------------------------
          // Create the same callback using the jextract output

          var myCallback = `hello_callback$my_cb`.allocate(
            {
              println("I GOT ME a simpler callback")
            },
            session
          )

          hams_h.hello_callback(myCallback)

        }


        // --------------------------------------------------------------
        // create enabled as an instance of the LogParam.enabled class then provide that as a reference into the allocate object of LogParam

        var myLogParamMS = LogParam.allocate(session)
        // println("LogParam is ${LogParam.sizeof()} bytes")

        var myEnabledMS = LogParam.enabled.allocate({myMemory: MemorySegment ->
          // println("i am the enabled func");
          true
        }, session)
        LogParam.`enabled$set`(myLogParamMS, myEnabledMS.address())

        var myLogMS = LogParam.log.allocate({myMemory: MemoryAddress ->
          var myExternCRecordMS = ExternCRecord.ofAddress(myMemory, session)
          var messageMS = ExternCRecord.`message$slice`(myExternCRecordMS)
          var module_pathMS = ExternCRecord.`module_path$slice`(myExternCRecordMS)

          println("${RustStr.`ptr$get`(module_pathMS).getUtf8String(0)}:${RustString.`ptr$get`(messageMS).getUtf8String(0)}")
;
        }, session)
        LogParam.`log$set`(myLogParamMS, myLogMS.address())

        var myFlushMS = LogParam.flush.allocate({
          println("")
        }, session)
        LogParam.`flush$set`(myLogParamMS, myFlushMS.address())


        LogParam.`level$set`(myLogParamMS, hams_h.ExternCLevelFilter_Info().toLong())

        hams_h.hams_logger_init(myLogParamMS)

  }
  fun checkMeOut() {
    println("I am checking my Hams out")
  }
}
