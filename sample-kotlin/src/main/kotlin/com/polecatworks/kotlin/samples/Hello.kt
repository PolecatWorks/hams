package com.polecatworks.kotlin.samples
import com.sun.jna.Native

fun main(args: Array<String>) {
    println("Hello, World")


    val ben = HamsJni()
    ben.testMe(3)
    val INSTANCE = Native.load("hams", CHams::class.java)
    INSTANCE.hello_world()
    // val ben2 = ben.foo(2);
    // println("Got a $ben2 as reply")
}
