
package com.polecatworks.kotlin.samples

import jdk.incubator.foreign.*
import java.lang.invoke.MethodHandles
import java.lang.invoke.MethodType


// https://blog.arkey.fr/2021/09/04/a-practical-look-at-jep-412-in-jdk17-with-libsodium/
// https://docs.oracle.com/en/java/javase/17/docs/api/jdk.incubator.foreign/jdk/incubator/foreign/package-summary.html
// https://stackoverflow.com/questions/69321128/how-to-call-a-c-function-from-java-17-using-jep-412-foreign-function-memory-a

internal object IntComparator {
    @JvmStatic
    fun intCompare(addr1: MemoryAddress, addr2: MemoryAddress): Int {
        println("I am going to compare two addresses")
        return MemoryAccess.getIntAtOffset(MemorySegment.globalNativeSegment(), addr1.toRawLongValue()) -
                MemoryAccess.getIntAtOffset(MemorySegment.globalNativeSegment(), addr2.toRawLongValue())
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


        var hello_world = CLinker.getInstance().downcallHandle(
            SymbolLookup.loaderLookup().lookup("hello_world").get(),
            MethodType.methodType(Void::class.javaPrimitiveType),
            FunctionDescriptor.ofVoid()
        )

        hello_world.invokeExact()

        var hello_node = CLinker.getInstance().downcallHandle(
            SymbolLookup.loaderLookup().lookup("hello_node").get(),
            MethodType.methodType(Int::class.javaPrimitiveType),
            FunctionDescriptor.of(CLinker.C_INT)
        )

        var mynode = hello_node.invokeExact() as Int
        println("hello_node replied with ${mynode}")





        // Start tp create a Java Call back function to log

        val helloCallbackHandle = MethodHandles.lookup().findStatic(
          HelloCallback::class.java,
          "helloCallback",
          MethodType.methodType(Void::class.javaPrimitiveType),
        )

        var scope = ResourceScope.newImplicitScope()

        val helloCallbackNativeSymbol = CLinker.getInstance().upcallStub(
          helloCallbackHandle,
          FunctionDescriptor.ofVoid(),
          scope,
        )

        // Find the function that will receive the callback

        var hello_callback = CLinker.getInstance().downcallHandle(
          SymbolLookup.loaderLookup().lookup("hello_callback").get(),
          MethodType.methodType(Void::class.javaPrimitiveType, MemoryAddress::class.java),
          FunctionDescriptor.ofVoid(CLinker.C_POINTER),
        )

        hello_callback.invokeExact(helloCallbackNativeSymbol)

        // Create a call back



        val intCompareHandle = MethodHandles.lookup().findStatic(
            IntComparator::class.java,
            "intCompare",
            MethodType.methodType(Int::class.javaPrimitiveType, MemoryAddress::class.java, MemoryAddress::class.java)
        )


        val comparFunc = CLinker.getInstance().upcallStub(
            intCompareHandle,
            FunctionDescriptor.of(CLinker.C_INT, CLinker.C_POINTER, CLinker.C_POINTER),
            scope
        )
        // TODO("Pass this upcallStub to a function to implement a callback")

//    var strlen = CLinker.getInstance().downcallHandle(
//
//        CLinker.systemLookup().lookup("strlen").get(),
//        MethodType.methodType(long.class, MemoryAddress.class),
//        FunctionDescriptor.of(CLinker.C_LONG, CLinker.C_POINTER)
//    );
//
//    try (var scope = ResourceScope.newConfinedScope()) {
//        var cString = CLinker.toCString("Hello", scope);
//        long len = (long)strlen.invokeExact(cString.address()); // 5
//    }
  }
  fun checkMeOut() {
    println("I am checking my Hams out")
  }
}
