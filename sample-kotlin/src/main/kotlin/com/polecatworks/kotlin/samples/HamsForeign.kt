
package com.polecatworks.kotlin.samples
import jdk.incubator.foreign.CLinker;
import jdk.incubator.foreign.FunctionDescriptor
import jdk.incubator.foreign.MemoryAddress
import jdk.incubator.foreign.SymbolLookup
import java.lang.invoke.MethodType


// https://blog.arkey.fr/2021/09/04/a-practical-look-at-jep-412-in-jdk17-with-libsodium/
// https://docs.oracle.com/en/java/javase/17/docs/api/jdk.incubator.foreign/jdk/incubator/foreign/package-summary.html

class HamsForeign constructor() {
    init {
        println("i am creating HamsForeign");
        System.loadLibrary("hams")

//    CLinker.getInstance().
//        var ab1 = CLinker.systemLookup().lookup("strlen").get()
//        var ab3 = FunctionDescriptor.of(CLinker.C_LONG, CLinker.C_POINTER)!!
//        var ab2 = MethodType.methodType(1L::class.java, MemoryAddress.class)
//        var strlen =  CLinker.getInstance().downcallHandle(
//                CLinker.systemLookup().lookup("strlen").get(),
//                MethodType.methodType(1L::class.java, MemoryAddress.class),
//                FunctionDescriptor.of(CLinker.C_LONG, CLinker.C_POINTER)!!
//            )

        // var lib_hello = CLinker.systemLookup().lookup("hello_world")
        var myLoader = SymbolLookup.loaderLookup()
       var lib_hello = myLoader.lookup("hello_world").get()



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
