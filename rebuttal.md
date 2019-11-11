We thank the reviewers for the thorough and thoughtful comments and will update the manuscript to reflect their suggestions. 

While we didn’t mention this in the paper, we will publish the sources and the data used in this manuscript. We also intend to support any attempts to replicate or enhance the study with assistance in understanding and running the code.

We will revise section 4.1 to include a precise description of the call graph algorithm using pseudocode.

**Review #61 AQ1: Can the authors provide examples of code impossible to write in Safe Rust?**

Unsafe is necessary for memory-mapped IO to cast an address to a Rust struct, and to use global variables to reference unique resources, in this case the COM1 port. An example from RedoxOS is:
```rust
impl SerialPort<Mmio<u32>> {

    pub **unsafe** fn new(base: usize) -> &'static mut SerialPort<Mmio<u32>> {

        &mut *(base as *mut Self)

    }

}

pub static COM1: Mutex<SerialPort<Pio<u8>>> = Mutex::new(SerialPort::<Pio<u8>>::new(0x3F8));

pub **unsafe** fn init() {

    COM1.lock().init();

}
```
**Review #61 AQ2: Is the use of call graph an overapproximation of unsafe use?**

Our methodology assumes that a method is possibly unsafe its body contains at least a call to a possibly unsafe method. Yes, it is possible that in some contexts a method call will never execute the path with unsafe code and more precise analysis combining data and control flow could yield more precise results. 

**Review #61 AQ3: Can Rust developers label an entirely safe piece of code as unsafe?**

Yes, it is possible, but if a block with entirely safe code is marked unsafe the compiler will print a warning. Functions with entirely safe code can be marked unsafe without a warning to restrict the context from which the function can be called. Also, methods marked unsafe in a trait must be defined as unsafe, even if the method only uses safe code.

**Review #61 AQ4: How much code marked unsafe is truly unsafe?**

An unsafe block is actually safe when it does not contain undefined behavior or violate memory safety, and this problem can not be solved precisely in general. Numerous analyses have been developed for C to determine if the code is free of a specific undefined behavior, but they suffer from false positives or false negatives, or both.

**Review #61 AQ5: How much code could be left as safe with better unsafe attribution? \
**The answer to this question depends on the definition of safety of unsafe code. We will implement a more precise analysis that removes the unnecessary unsafe for blocks and check if a declared unsafe function contains only safe operations. We will revise the manuscript to include the results of this analysis.

**Review #61 AQ6: Is “y” in foo() read only? **Yes, that is correct. 

**Review #61 AQ7: Is library 4 actually safe? If it is unsafe, why? \
**Library 4 depicts the difficulty in understanding unsafe in a codebase. It is possible the function is marked unsafe but calls no unsafe code. This would mean the programmer believes calling qux() is unsafe and should be restricted. The intent of LIbrary 4 is that casting “b” from one type to another is in general a memory unsafe operation and therefore the function is in fact unsafe.

**Review #61 AQ8: How often is transmute() used? \
**We will analyze the code and provide an answer to this question in the final version.

**Review #61 AQ9: Where does the source of sampling bias come from? \
**The crates included in our study are determined by the choice of the operating system and the Rust compiler version. While we covered a significant portion of the publicly available crates, we were not able to include all the Rust code available. We will revise section 5.2 for clarity.

**Review #61 B: **Although there are no specific questions for the authors, we will address some of the questions in the comments section. We already mentioned at the beginning of the rebuttal that we will revise section 4.1 and update the manuscript to   

We will investigate the suggestions to increase the precision of our analysis by distinguishing between commonly and rarely used APIs and to plot the density of unsafe functions. We will update the manuscript with the results of these analyses. 

In the 10-month gap in the RQ5, a new version of Rust has been released and the Unsafe Code Guidelines Working Group was created.

We intend to pursue as future work an analysis of the impact our recommendations have and to determine a small subset of unsafe functions that would have a big impact on the overall safety if they are determined to be actually safe.

**Review #61C AQ1: How much dead code is in the codebases? It may be easy to select just the sub-callgraphs starting from "main" to redo one of the statistics to address this threat to vulnerability. **

A fraction of the crates are purely libraries and do not have a main functions at all. We will address the question for the crates with a main function and we will revise the manuscript to include the findings.

**Review #61C AQ2: Have you reached out to the Mozilla Rust developers to assess their attitudes towards the "unsafeness"? If so, are they surprised by seeing your results?**

We plan to ask for feedback on our study from the Rust Language and Unsafe Guidelines teams and include it in the final version.


<!-- Docs to Markdown version 1.0β17 -->
