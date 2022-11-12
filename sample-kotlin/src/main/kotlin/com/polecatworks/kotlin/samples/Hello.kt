package com.polecatworks.kotlin.samples
import com.sun.jna.*

fun main(args: Array<String>) {
    println("Hello, World")


    val ben = HamsJni()
    ben.testMe(3)
    val INSTANCE = Native.load("hams", CHams::class.java)
    INSTANCE.hello_world()

    val ben2 = INSTANCE.hello_node()

    println("node is $ben2")
    // val ben2 = ben.foo(2);
    // println("Got a $ben2 as reply")

    var myRustStr = RustStr(Pointer(1),NativeLong(2L))
    println("RustStr = ${myRustStr.size()}")

    var myHamsForeign = HamsForeign()

    var myRustString = RustString(Pointer(1),NativeLong(2L), NativeLong(3))
    println("RustString = ${myRustString.size()}")
}
