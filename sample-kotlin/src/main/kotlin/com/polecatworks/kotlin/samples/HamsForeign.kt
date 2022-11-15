
package com.polecatworks.kotlin.samples

import jdk.incubator.foreign.*
import java.lang.invoke.MethodType


// https://blog.arkey.fr/2021/09/04/a-practical-look-at-jep-412-in-jdk17-with-libsodium/
// https://docs.oracle.com/en/java/javase/17/docs/api/jdk.incubator.foreign/jdk/incubator/foreign/package-summary.html

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
//
//
//
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
