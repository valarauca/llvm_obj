llvm_obj
---

#NOTICE THIS DOESNT WORK

#IF YOU COME ACROSS THIS ON 1-4-2017 IT HASNT BEEN TESTED

#A REFERENCE WAS WRITTEN DOCS CREATED BUT NOT DEBUGGED

#I NEED TO SPIN UP A LINUX VM

[Docs](https://valarauca.github.io/llvm_obj/llvm_obj/index.html)

This API is targetted at people generating LLVM-IR without the assitance
of the LLVM but still want to use the LLVM to generate object files.

This API *DOES NOT* make generating LLVM-IR easier. It really only
makes parsing IR and compiling that IR to object files easier.

To use this crate add the below to your projects `Cargo.toml`

```
[dependencies]
llvm_obj = "1.0.0"
```

####Example

`llvm_obj` offers a simple api to parse LLVM-IR in memory and write the
resulting object file to the file system. None of the core `llvm_sys`
types are exposed. 

```rust
extern llvm_obj;

///Error Enum
use llvm_obj::Fault;

///Simplest Example
///
///Generate code on the native platform (what ever this is ran on)
///Using the default options e.g.: no optimizations
pub fn write_object_file(name: &str, ir: &str, path: &str) -> Result<(),Fault> {

	//? isn't supported for custom results but that may get worked out for 1.15
	
	let mut mod = llvm_obj::LLVMod::new(name)?;
	module.parse(ir)?;
	let mut codegen = llvm_obj::Platform::new(None,None,None,None,None,None)?;
	module.write_object(path, &mut codegen)?;
	Ok(())
}
```

####API Expansions

This is a work in progress. It more or less works for what _I_ need. 

- [X] Works
- [ ] Version 1.5: Add assembly output
- [ ] Version 2.0: Add a boat load of Enums for Target and Features. Don't initialize unused backends
