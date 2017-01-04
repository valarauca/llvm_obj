/*
 * Copyright 2016 William Cody Laeder
 *
 * Licensed under the Apache License, Version 2.0 (the "License")
 * you may not use this file except in complicance with the License.
 * You may obtain a copy of the License at
 *
 *  http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either expressed or
 * implied.
 *
 * See the License for the specific language governing permissions and
 * limitations under the License
 */

//!High Level LLVM Compiler Wrapper
//!
//!This API is targetted at people generating LLVM-IR without the assitance
//!of the LLVM but still want to use the LLVM to generate object files.
//!
//!To use this crate add the below to your projects `Cargo.toml`
//!
//!```ignore
//![dependencies]
//!llvm_obj = "1.0.0"
//!```
//!The goal of this project is to make generating object files with the
//!`llvm_sys` wrapper relatively easy. 
//!
//!#Parse in memory
//!
//!llvm_obj offers a simple api to parse LLVM-IR in memory and write the
//!resulting object file to the file system.
//!
//!```ignore
//!extern crate llvm_obj;
//!
//!
//!let llvm_ir: &str = "";
//!let path: &str = "";
//!
//!
//!//build a new LLVMModule
//!let mut module = LLVMod::new("foobar").unwrap();
//!
//!//parse the IR
//!module.parse(llvm_ir).unwrap();
//!
//!//set codegen parameters
//!let mut codegen = Platform::new(None,None,None,None,None,None).unwrap();
//!
//!//write the object file
//!module.write_object(path,&mut codegen).unwrap();
//!```


extern crate llvm_sys;
use llvm_sys::core::*;
use llvm_sys::bit_reader::LLVMGetBitcodeModule;
use llvm_sys::LLVMModule;
use llvm_sys::prelude::*;
use llvm_sys::target::*;
use llvm_sys::target_machine::*;

use std::default::Default;
use std::ffi::{
    CString,
    CStr
};
use std::ptr::null_mut;


///CodeModel
///
///This correspondes 1:1 to the `LLVMCodeModel` enum
///
///For a reference see
///[link](http://rustdoc.taricorp.net/llvm-sys/llvm_sys/target_machine/enum.LLVMCodeModel.html)
pub enum CodeModel {
    Default,
    JIT,
    Small,
    Kernel,
    Medium,
    Large
}
impl Into<LLVMCodeModel> for CodeModel {

    /// Converts the non-verbose enum to the non-publically exported
    /// version
    #[inline(always)]
    fn into(self) -> LLVMCodeModel {
        match self {
            CodeModel::Default => LLVMCodeModel::LLVMCodeModelDefault,
            CodeModel::JIT => LLVMCodeModel::LLVMCodeModelJITDefault,
            CodeModel::Small => LLVMCodeModel::LLVMCodeModelSmall,
            CodeModel::Kernel => LLVMCodeModel::LLVMCodeModelKernel,
            CodeModel::Medium => LLVMCodeModel::LLVMCodeModelMedium,
            CodeModel::Large => LLVMCodeModel::LLVMCodeModelLarge
        }
    }
}
impl Default for CodeModel {
    /// This returns `Code::Default` very surprising
    #[inline(always)]
    fn default() -> CodeModel {
        CodeModel::Default
    }
}

///Reloc Mode
///
///This correspondes 1:1 to the `LLVMRelocMode` enum
///
///For a reference see
///[link](http://rustdoc.taricorp.net/llvm-sys/llvm_sys/target_machine/enum.LLVMRelocMode.html)
pub enum Reloc {
    Default,
    Static,
    PIC,
    DynamicNoPic,
}
impl Into<LLVMRelocMode> for Reloc {

    /// Converts the non-verbose enum to the non-publically exported
    /// version
    #[inline(always)]
    fn into(self) -> LLVMRelocMode {
        match self {
            Reloc::Default => LLVMRelocMode::LLVMRelocDefault,
            Reloc::Static => LLVMRelocMode::LLVMRelocStatic,
            Reloc::PIC => LLVMRelocMode::LLVMRelocPIC,
            Reloc::DynamicNoPic => LLVMRelocMode::LLVMRelocDynamicNoPic,
        }
    }
}
impl Default for Reloc {
    /// This returns `Code::Default` very surprising
    #[inline(always)]
    fn default() -> Self {
        Reloc::Default
    }
}

///Opt Mode
///
///This correspondes 1:1 to the `LLVMGenOptLevel` enum
///
///For a reference see
///[link](http://rustdoc.taricorp.net/llvm-sys/llvm_sys/target_machine/enum.LLVMGenOptLevel.html)
pub enum Opt {
    None,
    Less,
    Default,
    Aggressive
}
impl Into<LLVMCodeGenOptLevel> for Opt {

    /// Converts the non-verbose enum to the non-publically exported
    /// version
    #[inline(always)]
    fn into(self) -> LLVMCodeGenOptLevel {
        match self {
            Opt::Default => LLVMCodeGenOptLevel::LLVMCodeGenLevelDefault,
            Opt::None => LLVMCodeGenOptLevel::LLVMCodeGenLevelNone,
            Opt::Less => LLVMCodeGenOptLevel::LLVMCodeGenLevelLess,
            Opt::Aggressive => LLVMCodeGenOptLevel::LLVMCodeGenLevelAggressive,
        }
    }
}
impl Default for Opt {
    /// This returns `Opt::None`
    ///
    /// I'm writing this to make an assembler so this is _my_ default.
    #[inline(always)]
    fn default() -> Self {
        Opt::None
    }
}


///Target Platfrom
///
///Wraps `LLVMTargetMachineRef`. This represents _everything_ to do with
///the target platfrom. What CPU we're targeting, what features that CPU
///has, how aggressive the codegen is, what we're optimizing for, is the
///binary static or can it be relocated.
pub struct Platform {
    target: LLVMTargetMachineRef
}
impl Platform {
    
    /// Tell the LLVM WTF we're targetting!
    ///
    /// There is a ton to unpack here so going arg by arg. 
    ///
    /// * `triple` - Target-Triple-String. If `None` this will result to
    /// the platform executing the library. 
    /// * `cpu` - Argument normally passed to ` llc -mcpu=a1`. This is for
    /// optimization purposes. If no argument is passed the arg will be
    /// `"generic"`. 
    /// * `features` - Argument normally passed to `llc -mattr=a1,+a2,-a3`
    /// If `None` no features are used.
    /// * `codegen` - Optimization level. See docs for `Code`. 
    /// If `None` NO optimization will be done. 
    /// * `relocmodel` - Specifies if code is position independent or not.
    /// See docs for `Reloc`. If `None` the value `LLVMRelocDefault` is
    /// used.
    /// * `codemodel` - What code output is being generated. 
    /// If `None` the value `LLVMCodeModelDefault` is used. See docs for
    /// `CodeModel`.
    pub fn new(
    triple: Option<&str>,
    cpu: Option<&str>,
    features: Option<&str>,
    codegen: Option<CodeModel>,
    relocmodel: Option<Reloc>,
    codemodel: Option<Opt>) -> Result<Self,Fault> {
	use std::mem;        

        let trip = match get_triple(triple) {
            Ok(x) => x,
            Err(e) => return Err(e)
        };
        let mut target: *mut LLVMTargetRef = null_mut();
        let mut err = null_mut();
        unsafe{
            let trip_ptr = unsafe{trip.as_ptr() as *const _};
            LLVMGetTargetFromTriple(trip_ptr,target,err);
        }
        if target.is_null() {
            return Err(build_fault(err));
        }
        let cpu = match cpu {
            Option::Some(c) => CString::new(c).unwrap(),
            Option::None => CString::new("generic").unwrap()
        };
        let features = match features {
            Option::Some(f) => CString::new(f).unwrap(),
            Option::None => CString::new("").unwrap()
        };
        let opt: LLVMCodeGenOptLevel = match codemodel {
            Option::Some(x) => x,
            Option::None => Opt::default()
        }.into();
        let reloc: LLVMRelocMode = match relocmodel {
            Option::Some(x) => x,
            Option::None => Reloc::default()
        }.into();
        let model: LLVMCodeModel = match codegen {
            Option::Some(x) => x,
            Option::None => CodeModel::default()
        }.into();
        unsafe{
            Ok(Platform{
                target: LLVMCreateTargetMachine(
                    *target,
                    trip.as_ptr() as *const _,
                    cpu.as_ptr() as *const _,
                    features.as_ptr() as *const _,
                    opt,
                    reloc,
                    model)
            })
        }
    }
}
impl Drop for Platform {
    fn drop(&mut self) {
        unsafe {
            LLVMDisposeTargetMachine(self.target);
        }
    }
}

/// Get Target-Triple
///
/// If `Option::None` is passed it willc all the LLVM to fetch the triple
/// of the platform this code is executing on.
#[inline]
pub fn get_triple(x: Option<&str>) -> Result<CString,Fault> {
    match x {
        Option::Some(y) => match CString::new(y) {
            Ok(z) => Ok(z),
            Err(_) => return Err(Fault::NullPtr)
        },
        Option::None => unsafe {
            let a = LLVMGetDefaultTargetTriple();
            let trip = CStr::from_ptr(a as *const _).to_owned();
            LLVMDisposeMessage(a);
            Ok(trip)
        }
    }
}

///Errors that can occur when working LLVM API
pub enum Fault {
    /// Either a null ptr was passed to this API or this API received a
    /// null ptr from the LLVM
    NullPtr,
    /// LLVM Action returned an error.
    Err(String),
    /// An Error occured. But the LLVM's error ptr is null.
    ErrNull,
    /// An Error occured. But the error's text isn't UTF8 compatible
    /// This should _never_ happen as the LLVM only works in ASCII
    /// but it is included for completeness.
    ErrUTF8(CString)
}
/*
 * Converts an LLVM Error message to the corresponding
 * Fault value
 */
fn build_fault<T>(x: *mut T) -> Fault {
    use std::str;

    if x.is_null() {
        Fault::ErrNull
    } else {
        let cstr = unsafe{CStr::from_ptr(x as *const _)};
        match str::from_utf8(cstr.to_bytes()) {
            Ok(x) => Fault::Err(x.to_string()),
            Err(_) => Fault::ErrUTF8(cstr.to_owned())
        }
    }
}

///An LLVM Module
///
///Memory representation of an LLVMModule pointer with associated
///data
pub struct LLVMod {
    module: *mut LLVMModule,
    data: Vec<CString>
}
impl LLVMod {
    /// Converts the internally held LLVM-IR into a CString
    ///
    /// Returns `Err(_)` if the LLVM-IR cannot be
    /// converted from a null ptr is returned from the LLVM
    pub fn to_cstring(&self) -> Result<CString,Fault> {
        unsafe{
            let ir = LLVMPrintModuleToString(self.module);
            let ir_cstr = CStr::from_ptr(ir as *const _);
            let ret = match CString::new(ir_cstr.to_bytes()) {
                Ok(x) => Result::Ok(x),
                Err(_) => Result::Err(Fault::NullPtr)
            };
            LLVMDisposeMessage(ir);
            ret
        }
    }
    /// Adds a new string
    ///
    /// The returned Pointer is owned by `self` so it'll
    /// say allocated as long as `self` does.
    fn add_str(&mut self, s: &str) -> Result<*mut i8, Fault> {
        match CString::new(s) {
            Ok(x) => {
                let ptr = x.as_ptr() as *mut _;
                self.data.push(x);
                Ok(ptr)
            },
            Err(_) => Err(Fault::NullPtr)
        }
    }
    /// Build a module
    pub fn new(name: &str) -> Result<LLVMod,Fault> {
        let name = match CString::new(name) {
            Ok(x) => x,
            Err(_) => return Err(Fault::NullPtr)
        };
        let ptr = name.to_bytes_with_nul().as_ptr() as *const _;
        let module = unsafe{LLVMModuleCreateWithName(ptr)};
        Ok(LLVMod {
            module: module,
            data: vec![name]
        })
    }
    /// Parse LLVM-IR
    pub fn parse(&mut self, ir: &str) -> Result<(),Fault> {
        use std::mem;

	let len = ir.as_bytes().len();
	let ir = match self.add_str(ir) {
            Ok(x) => x,
            Err(e) => return Err(e)
        };
        unsafe{
	    let val_start: usize = mem::transmute(ir);
	    let val_end: usize = val_start + len;
	let start: *mut i8 = mem::transmute(val_start);
	let end: *mut i8 = mem::transmute(val_end);
	let buf = LLVMCreateMemoryBufferWithMemoryRange(start,len,end,0);
	if buf.is_null() {
		return Err(Fault::NullPtr);
	}
            let mut err = null_mut();
            let flag = LLVMGetBitcodeModule(
                buf,
                &mut self.module,
                err);
            if flag != 0 {
                return Err(build_fault(err));
            } else {
                Ok(())
            }
        }
    }
    /// Write Object File
    ///
    /// `Platform` is used to specify the code generation options
    pub fn write_object(&mut self, path: &str, target: &mut Platform)
    -> Result<(),Fault> {
        unsafe{
            let path = try!(self.add_str(path)); 
            LLVM_InitializeAllTargetInfos();
            LLVM_InitializeAllTargets();
            LLVM_InitializeAllTargetMCs();
            LLVM_InitializeAllAsmParsers();
            LLVM_InitializeAllAsmPrinters();
            let mut err = null_mut();
            let flag = LLVMTargetMachineEmitToFile(
                target.target,
                self.module,
                path,
                LLVMCodeGenFileType::LLVMObjectFile,
                &mut err
            );
            if flag != 0 {
                Err(build_fault(err))
            } else {
                Ok(())
            }
        }
    }
}
impl Drop for LLVMod {
    
    /// Drops the Module and all related CStrings
    fn drop(&mut self) {
        unsafe{
            LLVMDisposeModule(self.module);
        }
    }
}
                

