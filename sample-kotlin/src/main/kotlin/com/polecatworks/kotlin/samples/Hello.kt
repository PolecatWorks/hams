package com.polecatworks.kotlin.samples
import com.sun.jna.*
import MyJava

fun main(args: Array<String>) {
    println("Hello, World")

    var myJava = MyJava()
    myJava.howdy()

    val ben = HamsJni()
    ben.testMe(3)
    val instance = Native.load("hams", CHams::class.java)
    instance.hello_world()

    // val ben2 = INSTANCE.hello_node()

    // println("node is $ben2")
    // // val ben2 = ben.foo(2);
    // // println("Got a $ben2 as reply")

    // var myRustStr = RustStr(Pointer(1),NativeLong(2L))
    // println("RustStr = ${myRustStr.size()}")

    //  var myHamsForeign = HamsForeign()

    // myHamsForeign.checkMeOut()

    // var myRustString = RustString(Pointer(1),NativeLong(2L), NativeLong(3))
    // println("RustString = ${myRustString.size()}")
}
