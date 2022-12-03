
package com.polecatworks.kotlin.samples

import java.lang.foreign.*
import java.lang.invoke.MethodHandles
import java.lang.invoke.MethodType
import java.nio.ByteOrder
import com.polecatworks.hams.hams_h
import com.polecatworks.hams.LogParam

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

        // Start to create a Java Call back function to log

        val helloCallbackDescriptor = FunctionDescriptor.ofVoid()
        val helloCallbackHandle = MethodHandles.lookup().findStatic(
          HelloCallback::class.java,
          "helloCallback",
          Linker.upcallType(helloCallbackDescriptor)
        )

        var session = MemorySession.openImplicit()

        val helloCallbackNativeSymbol = linker.upcallStub(
          helloCallbackHandle, helloCallbackDescriptor, session
        )

        println("About to make callback [${helloCallbackNativeSymbol} ]")
        hams_h.hello_callback(helloCallbackNativeSymbol)

        // create enabled as an instance of the LogParam.enabled class then provide that as a reference into the allocate object of LogParam

        var myLogParam = LogParam.allocate(session)
        println("LogParam is ${LogParam.sizeof()} bytes")


        class MyEnabled : LogParam.enabled {
          override fun apply(myMemory: MemorySegment): Boolean {
            println("i am the enabled func")
            return true
          }
        }

        var myEnabled = LogParam.enabled.allocate(MyEnabled(), session)
        LogParam.`enabled$set`(myLogParam, myEnabled.address())

        class MyLog: LogParam.log {

          override fun apply(myMemory: MemoryAddress) {
            println("i am the log func")
          }
        }

        // var myLog = LogParam.log.allocate(MyLog(), session)
        // LogParam.`log$set`(myLogParam, myLog.address())

        class MyFlush: LogParam.flush {
          override fun apply() {
            println("i am the flush func")
          }
        }

        var myFlush = LogParam.flush.allocate(MyFlush(), session)
        LogParam.`flush$set`(myLogParam, myFlush.address())


        LogParam.`level$set`(myLogParam, hams_h.ExternCLevelFilter_Info().toLong())

        println("About to register LogParam [${myEnabled.address()} ]")

        hams_h.hams_logger_init(myLogParam)



      /*
        // Find the function that will receive the callback

        var hello_callback = Linker.getInstance().downcallHandle(
          SymbolLookup.loaderLookup().lookup("hello_callback").get(),
          MethodType.methodType(Void::class.javaPrimitiveType, MemoryAddress::class.java),
          FunctionDescriptor.ofVoid(Linker.C_POINTER),
        )
        // Try out the basic callback
        hello_callback.invokeExact(helloCallbackNativeSymbol)

//        const uint8_t *ptr;
//        uintptr_t len;
        var RustStr = MemoryLayout.structLayout(
            MemoryLayout.valueLayout(64, ByteOrder.nativeOrder()).withName("ptr"),
            MemoryLayout.valueLayout(64, ByteOrder.nativeOrder()).withName("len"),
        ).withName("RustStr")

        var RustString = MemoryLayout.structLayout(
            MemoryLayout.valueLayout(64, ByteOrder.nativeOrder()).withName("ptr"),
            MemoryLayout.valueLayout(64, ByteOrder.nativeOrder()).withName("cap"),
            MemoryLayout.valueLayout(64, ByteOrder.nativeOrder()).withName("len"),
        ).withName("RustString")

        var LogParam = MemoryLayout.structLayout(
            MemoryLayout.valueLayout(64, ByteOrder.nativeOrder()).withName("enabled"),
            MemoryLayout.valueLayout(64, ByteOrder.nativeOrder()).withName("log"),
            MemoryLayout.valueLayout(64, ByteOrder.nativeOrder()).withName("flush"),
            MemoryLayout.valueLayout(64, ByteOrder.nativeOrder()).withName("level"),
        ).withName("LogParam")

        var ExternCMetadata = MemoryLayout.structLayout(
            MemoryLayout.valueLayout(64, ByteOrder.nativeOrder()).withName("level"),
            RustStr.withName("target"),
        ).withName("ExternCMetadata")


        var ExternCRecord = MemoryLayout.structLayout(
            ExternCMetadata.withName("metadata"),
            RustString.withName("message"),
            RustStr.withName("module_path"),
            RustStr.withName("file"),
            MemoryLayout.valueLayout(64, ByteOrder.nativeOrder()).withName("line"),
        ).withName("ExternCRecord")




        println("RustStr is ${RustStr.bitSize()}")
        println("RustString is ${RustString.bitSize()}")
        println("LogParam is ${LogParam.bitSize()}")
        println("ExternCMetadata is ${ExternCMetadata.bitSize()}, ${ExternCMetadata.toString()}")
        println("ExternCRecord is ${ExternCRecord.bitSize()}, ${ExternCRecord.toString()}")




        // Create a call back



        val intCompareHandle = MethodHandles.lookup().findStatic(
            IntComparator::class.java,
            "intCompare",
            MethodType.methodType(Int::class.javaPrimitiveType, MemoryAddress::class.java, MemoryAddress::class.java)
        )


        val comparFunc = Linker.getInstance().upcallStub(
            intCompareHandle,
            FunctionDescriptor.of(Linker.C_INT, Linker.C_POINTER, Linker.C_POINTER),
            scope
        )
        // TODO("Pass this upcallStub to a function to implement a callback")
   */

//    var strlen = Linker.getInstance().downcallHandle(
//
//        Linker.systemLookup().lookup("strlen").get(),
//        MethodType.methodType(long.class, MemoryAddress.class),
//        FunctionDescriptor.of(Linker.C_LONG, Linker.C_POINTER)
//    );
//
//    try (var scope = ResourceScope.newConfinedScope()) {
//        var cString = Linker.toCString("Hello", scope);
//        long len = (long)strlen.invokeExact(cString.address()); // 5
//    }
  }
  fun checkMeOut() {
    println("I am checking my Hams out")
  }
}
