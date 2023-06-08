// Interpret bytecode

use crate::objects::exceptionobject::{self, methodnotdefinedexc_from_str};
use crate::objects::{dictobject, mhash};
use crate::parser::Position;
use crate::trc::Trc;
use crate::{
    compiler::{Bytecode, CompilerInstruction, CompilerRegister},
    fileinfo::FileInfo,
    none_from,
    objects::{
        boolobject, fnobject, intobject, listobject, noneobject, utils::object_repr_safe, Object,
    },
    stats, TimeitHolder,
};
use colored::Colorize;
use std::marker::PhantomData;
use std::ops::DerefMut;
use std::time::Instant;

#[derive(Clone, PartialEq, Eq)]
pub struct Namespaces<'a> {
    variables: Vec<Vec<Option<Object<'a>>>>,
    _marker: PhantomData<&'a ()>,
}

pub const MIN_INT_CACHE: i128 = -5;
pub const MAX_INT_CACHE: i128 = 256;
pub const INT_CACHE_SIZE: i128 = MAX_INT_CACHE - MIN_INT_CACHE;

#[derive(Clone)]
pub struct SingletonCache<'a> {
    pub int_cache: [Option<Object<'a>>; INT_CACHE_SIZE as usize],
    pub bool_cache: (Option<Object<'a>>, Option<Object<'a>>),
    pub none_singleton: Option<Object<'a>>,
    _marker: PhantomData<&'a ()>,
}

#[derive(Clone)]
pub struct VM<'a> {
    pub types: Trc<hashbrown::HashMap<String, Object<'a>>>,
    pub interpreters: Vec<Trc<Interpreter<'a>>>,
    pub namespaces: Trc<Namespaces<'a>>,
    info: FileInfo<'a>,
    pub cache: SingletonCache<'a>,
}

impl<'a> VM<'a> {
    #[inline(always)]
    pub fn get_type(&self, name: &str) -> Object<'a> {
        return self.types.get(name).expect("Type not found").clone();
    }
    pub fn add_type(mut this: Trc<Self>, name: &str, value: Object<'a>) {
        (*this.types).insert(name.to_string(), value);
    }
}

impl<'a> Eq for VM<'a> {}

impl<'a> PartialEq for VM<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.types == other.types
            && self.interpreters == other.interpreters
            && self.namespaces == other.namespaces
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct Interpreter<'a> {
    frames: Vec<Frame<'a>>,
    types: Trc<hashbrown::HashMap<String, Object<'a>>>,
    namespaces: Trc<Namespaces<'a>>,
    vm: Trc<VM<'a>>,
}

#[derive(Clone, PartialEq, Eq)]
struct Frame<'a> {
    registers: Vec<Object<'a>>,
}

macro_rules! pop_frame {
    ($interp:expr) => {{
        (*$interp.namespaces).variables.pop();
        $interp.frames.pop();
    }};
}

macro_rules! add_frame {
    ($interp:expr, $n_registers:expr, $n_vars:expr) => {{
        let mut variables = Vec::new();
        for _ in 0..$n_vars {
            variables.push(None);
        }
        (*$interp.namespaces).variables.push(variables);
            
        let mut registers = Vec::new();
        for _ in 0..$n_registers {
            registers.push(none_from!($interp.vm.clone()));
        }
        $interp.frames.push(Frame { registers })
    }};
}

impl<'a> VM<'a> {
    pub fn new(info: FileInfo<'a>) -> VM<'a> {
        let singleton = SingletonCache {
            int_cache: intobject::init_cache(),
            bool_cache: (None, None),
            none_singleton: None,
            _marker: PhantomData,
        };
        VM {
            types: Trc::new(hashbrown::HashMap::new()),
            interpreters: Vec::new(),
            namespaces: Trc::new(Namespaces {
                variables: Vec::new(),
                _marker: PhantomData,
            }),
            info,
            cache: singleton,
        }
    }

    pub fn init_cache(this: Trc<Self>) {
        let int_cache_arr_ref = &(*this).cache.int_cache;
        let ptr = int_cache_arr_ref as *const [Option<Object>; INT_CACHE_SIZE as usize]
            as *mut [Option<Object>; INT_CACHE_SIZE as usize];
        intobject::generate_cache(this.get_type("int"), ptr);

        let bool_cache_tup_ref = &(*this).cache.bool_cache;
        let ptr = bool_cache_tup_ref as *const (Option<Object>, Option<Object>)
            as *mut (Option<Object>, Option<Object>);
        boolobject::generate_cache(this.get_type("bool"), ptr);

        let none_obj_ref = &(*this).cache.none_singleton;
        let ptr = none_obj_ref as *const Option<Object> as *mut Option<Object>;
        noneobject::generate_cache(this.get_type("NoneType"), ptr);
    }

    pub fn execute(mut this: Trc<Self>, bytecode: Trc<Bytecode<'a>>) -> Object<'a> {
        let interpreter =
            Interpreter::new(this.types.clone(), this.namespaces.clone(), this.clone());

        (*this).interpreters.push(Trc::new(interpreter));
        let last = this.deref_mut().interpreters.last_mut().unwrap();
        return last.run_interpreter(bytecode);
    }

    pub fn execute_timeit(
        mut this: Trc<Self>,
        bytecode: Trc<Bytecode<'a>>,
        timeit: &mut TimeitHolder,
    ) -> Object<'a> {
        //See bench.rs, this is a verys similar implementation (pub fn iter<T, F>(inner: &mut F) -> stats::Summary)

        let samples = &mut [0f64; 50];

        //Get initial result
        let mut res = (this.deref_mut().interpreters.last_mut().unwrap()).run_interpreter(bytecode.clone());

        for p in &mut *samples {
            let start = Instant::now();
            for _ in 0..5 {
                res = (this.deref_mut().interpreters.last_mut().unwrap()).run_interpreter(bytecode.clone());
            }
            let delta = start.elapsed().as_nanos();
            let time = if (delta as i128 / 5_i128) - (timeit.baseline as i128) < 0 {
                0
            } else {
                delta / 5 - timeit.baseline
            };
            *p = time as f64;
        }

        stats::winsorize(samples, 5.0);

        let sum: f64 = samples.iter().sum();

        timeit.time = sum / samples.len() as f64;

        res
    }

    pub fn execute_vars(
        mut this: Trc<Self>,
        bytecode: Trc<Bytecode<'a>>,
        vars: hashbrown::HashMap<&i128, Object<'a>>,
    ) -> Object<'a> {
        let interpreter =
        Interpreter::new(this.types.clone(), this.namespaces.clone(), this.clone());
        (*this).interpreters.push(Trc::new(interpreter));

        let res = (this.deref_mut().interpreters.last_mut().unwrap()).run_interpreter_vars(bytecode, vars);
        res
    }

    pub fn terminate(_: Trc<Self>) -> ! {
        //Clean up child threads here
        std::process::exit(1);
    }
}

macro_rules! load_register {
    ($this:expr, $last:expr, $namespaces:expr, $bytecode:expr, $i:expr, $register:expr) => {
        match $register {
            CompilerRegister::R(v) => $last.registers[v as usize].clone(),
            CompilerRegister::V(v) => match &$namespaces.variables.last().unwrap()[v as usize] {
                Some(v) => v.clone(),
                None => {
                    let pos = $bytecode
                        .positions
                        .get($i)
                        .expect("Instruction out of range");
                    let exc = exceptionobject::nameexc_from_str(
                        $this.vm.clone(),
                        &format!(
                            "Name '{}' not defined",
                            $bytecode.names.get(&($i as i32)).unwrap()
                        ),
                        pos.0,
                        pos.1,
                    );
                    $this.raise_exc_pos(exc, pos.0, pos.1);
                }
            },
        }
    };
}

macro_rules! store_register {
    ($last:expr, $namespaces:expr, $register:expr, $value:expr) => {
        match $register {
            CompilerRegister::R(v) => $last.registers[v as usize] = $value,
            CompilerRegister::V(v) => (*$namespaces).variables.last_mut().unwrap()[v as usize] = Some($value),
        }
    };
}

impl<'a> Interpreter<'a> {
    pub fn new(
        types: Trc<hashbrown::HashMap<String, Object<'a>>>,
        namespaces: Trc<Namespaces<'a>>,
        vm: Trc<VM<'a>>,
    ) -> Interpreter<'a> {
        Interpreter {
            frames: Vec::new(),
            types,
            namespaces,
            vm,
        }
    }

    #[allow(dead_code)]
    pub fn raise_exc(&self, exc_obj: Object<'a>) -> ! {
        let exc = exc_obj
            .internals
            .get_exc()
            .expect("Expected exc internal value");
        self.raise_exc_pos(exc_obj, exc.start, exc.end);
    }

    fn raise_exc_pos(&self, exc_obj: Object<'a>, start: Position, end: Position) -> ! {
        let header: String = match object_repr_safe(exc_obj) {
            crate::objects::MethodValue::Some(v) => v,
            _ => {
                unimplemented!()
            }
        };
        let location: String = format!(
            "{}:{}:{}",
            self.vm.info.name,
            start.line + 1,
            start.startcol + 1
        );
        println!("{}", header.red().bold());
        println!("{}", location.red());
        let lines = Vec::from_iter(self.vm.info.data.split(|num| *num as char == '\n'));

        let snippet: String = format!(
            "{}",
            String::from_utf8(
                lines
                    .get(start.line)
                    .expect("Line index out of range")
                    .to_vec()
            )
            .expect("utf8 conversion failed")
            .blue()
        );
        let mut arrows: String = String::new();
        for idx in 0..snippet.len() {
            if idx >= start.startcol && idx < end.endcol {
                arrows += "^";
            } else {
                arrows += " ";
            }
        }
        let linestr = (start.line + 1).to_string().blue().bold();
        println!("{} | {}", linestr, snippet);
        println!("{} | {}", " ".repeat(linestr.len()), arrows.green());

        //Should this happen??
        VM::terminate(self.vm.clone());
    }

    pub fn run_interpreter_vars(
        &mut self,
        bytecode: Trc<Bytecode<'a>>,
        vars: hashbrown::HashMap<&i128, Object<'a>>,
    ) -> Object<'a> {
        add_frame!(
            self,
            bytecode.n_registers as usize,
            bytecode.n_variables as usize
        );
        
        for (i, var) in (*self.namespaces)
            .variables
            .last_mut()
            .unwrap()
            .iter_mut()
            .enumerate()
        {
            if vars.get(&(i as i128)).is_some() {
                *var = Some(vars.get(&(i as i128)).unwrap().clone());
            }
        }

        self.run_interpreter_raw(bytecode)
    }

    pub fn run_interpreter(&mut self, bytecode: Trc<Bytecode<'a>>) -> Object<'a> {
        if !bytecode.instructions.is_empty() {
            add_frame!(
                self,
                bytecode.n_registers as usize,
                bytecode.n_variables as usize
            );
            return self.run_interpreter_raw(bytecode);
        }
        none_from!(self.vm)
    }

    #[inline]
    pub fn run_interpreter_raw(&mut self, bytecode: Trc<Bytecode<'a>>) -> Object<'a> {
        for (i, instruction) in bytecode.instructions.iter().enumerate() {
            match instruction {
                //Constant loading
                CompilerInstruction::LoadConst { index, register } => {
                    store_register!(
                        self.frames.last_mut().expect("No frames"),
                        self.namespaces,
                        *register,
                        bytecode
                            .consts
                            .get(*index)
                            .expect("Bytecode consts index out of range")
                            .clone()
                    );
                }

                //Binary operations
                CompilerInstruction::BinaryAdd { a, b, result } => {
                    let last = self.frames.last_mut().expect("No frames");
                    if load_register!(self, last, self.namespaces, bytecode, i, *a)
                        .add
                        .is_none()
                    {
                        let pos = bytecode.positions.get(i).expect("Instruction out of range");
                        let exc = methodnotdefinedexc_from_str(
                            self.vm.clone(),
                            &format!(
                                "Method 'add' is not defined for '{}' type",
                                load_register!(self, last, self.namespaces, bytecode, i, *a)
                                    .typename
                            ),
                            pos.0,
                            pos.1,
                        );
                        self.raise_exc(exc);
                    }

                    let res = (load_register!(self, last, self.namespaces, bytecode, i, *a)
                        .add
                        .expect("Method is not defined"))(
                        load_register!(self, last, self.namespaces, bytecode, i, *a).clone(),
                        load_register!(self, last, self.namespaces, bytecode, i, *b).clone(),
                    );
                    maybe_handle_exception!(self, res, bytecode, i);
                    store_register!(last, self.namespaces, *result, res.unwrap());
                }
                CompilerInstruction::BinarySub { a, b, result } => {
                    let last = self.frames.last_mut().expect("No frames");
                    if load_register!(self, last, self.namespaces, bytecode, i, *a)
                        .sub
                        .is_none()
                    {
                        let pos = bytecode.positions.get(i).expect("Instruction out of range");
                        let exc = methodnotdefinedexc_from_str(
                            self.vm.clone(),
                            &format!(
                                "Method 'sub' is not defined for '{}' type",
                                load_register!(self, last, self.namespaces, bytecode, i, *a)
                                    .typename
                            ),
                            pos.0,
                            pos.1,
                        );
                        self.raise_exc(exc);
                    }
                    let res = (load_register!(self, last, self.namespaces, bytecode, i, *a)
                        .sub
                        .expect("Method is not defined"))(
                        load_register!(self, last, self.namespaces, bytecode, i, *a).clone(),
                        load_register!(self, last, self.namespaces, bytecode, i, *b).clone(),
                    );
                    maybe_handle_exception!(self, res, bytecode, i);
                    store_register!(last, self.namespaces, *result, res.unwrap());
                }
                CompilerInstruction::BinaryMul { a, b, result } => {
                    let last = self.frames.last_mut().expect("No frames");
                    if load_register!(self, last, self.namespaces, bytecode, i, *a)
                        .mul
                        .is_none()
                    {
                        let pos = bytecode.positions.get(i).expect("Instruction out of range");
                        let exc = methodnotdefinedexc_from_str(
                            self.vm.clone(),
                            &format!(
                                "Method 'mul' is not defined for '{}' type",
                                load_register!(self, last, self.namespaces, bytecode, i, *a)
                                    .typename
                            ),
                            pos.0,
                            pos.1,
                        );
                        self.raise_exc(exc);
                    }

                    let res = (load_register!(self, last, self.namespaces, bytecode, i, *a)
                        .mul
                        .expect("Method is not defined"))(
                        load_register!(self, last, self.namespaces, bytecode, i, *a).clone(),
                        load_register!(self, last, self.namespaces, bytecode, i, *b).clone(),
                    );
                    maybe_handle_exception!(self, res, bytecode, i);
                    store_register!(last, self.namespaces, *result, res.unwrap());
                }
                CompilerInstruction::BinaryDiv { a, b, result } => {
                    let last = self.frames.last_mut().expect("No frames");
                    if load_register!(self, last, self.namespaces, bytecode, i, *a)
                        .div
                        .is_none()
                    {
                        let pos = bytecode.positions.get(i).expect("Instruction out of range");
                        let exc = methodnotdefinedexc_from_str(
                            self.vm.clone(),
                            &format!(
                                "Method 'div' is not defined for '{}' type",
                                load_register!(self, last, self.namespaces, bytecode, i, *a)
                                    .typename
                            ),
                            pos.0,
                            pos.1,
                        );
                        self.raise_exc(exc);
                    }

                    let res = (load_register!(self, last, self.namespaces, bytecode, i, *a)
                        .div
                        .expect("Method is not defined"))(
                        load_register!(self, last, self.namespaces, bytecode, i, *a).clone(),
                        load_register!(self, last, self.namespaces, bytecode, i, *b).clone(),
                    );
                    maybe_handle_exception!(self, res, bytecode, i);
                    store_register!(last, self.namespaces, *result, res.unwrap());
                }

                //Unary operations
                CompilerInstruction::UnaryNeg { a, result } => {
                    let last = self.frames.last_mut().expect("No frames");

                    if load_register!(self, last, self.namespaces, bytecode, i, *a)
                        .neg
                        .is_none()
                    {
                        let pos = bytecode.positions.get(i).expect("Instruction out of range");
                        let exc = methodnotdefinedexc_from_str(
                            self.vm.clone(),
                            &format!(
                                "Method 'neg' is not defined for '{}' type",
                                load_register!(self, last, self.namespaces, bytecode, i, *a)
                                    .typename
                            ),
                            pos.0,
                            pos.1,
                        );
                        self.raise_exc(exc);
                    }

                    let res = (load_register!(self, last, self.namespaces, bytecode, i, *a)
                        .neg
                        .expect("Method is not defined"))(
                        load_register!(self, last, self.namespaces, bytecode, i, *a).clone(),
                    );
                    maybe_handle_exception!(self, res, bytecode, i);
                    store_register!(last, self.namespaces, *result, res.unwrap());
                }

                //Register manipulation
                CompilerInstruction::CopyRegister { from, to } => {
                    let last = self.frames.last_mut().expect("No frames");
                    store_register!(
                        last,
                        self.namespaces,
                        *to,
                        load_register!(self, last, self.namespaces, bytecode, i, *from)
                    );
                }

                //Functions, arguments
                CompilerInstruction::MakeFunction {
                    nameidx,
                    argsidx,
                    codeidx,
                    idxsidx,
                    out,
                } => {
                    let last = self.frames.last_mut().expect("No frames");
                    let code = bytecode
                        .consts
                        .get(*codeidx)
                        .expect("Bytecode consts index out of range")
                        .clone();
                    let args = bytecode
                        .consts
                        .get(*argsidx)
                        .expect("Bytecode consts index out of range")
                        .clone();
                    let name = bytecode
                        .consts
                        .get(*nameidx)
                        .expect("Bytecode names index out of range")
                        .clone();
                    let indices = bytecode
                        .consts
                        .get(*idxsidx)
                        .expect("Bytecode names index out of range")
                        .clone();
                    let func = fnobject::fn_from(
                        self.vm.clone(),
                        code,
                        args.internals
                            .get_arr()
                            .expect("Expected arr internal value")
                            .clone(),
                        indices
                            .internals
                            .get_arr()
                            .expect("Expected arr internal value")
                            .clone(),
                        name.internals
                            .get_str()
                            .expect("Expected str internal value")
                            .clone(),
                    );
                    store_register!(last, self.namespaces, *out, func);
                }
                CompilerInstruction::Call {
                    callableregister,
                    result,
                    arg_registers,
                } => {
                    let last = self.frames.last_mut().expect("No frames");
                    let callable =
                        load_register!(self, last, self.namespaces, bytecode, i, *callableregister);
                    let mut args = Vec::new();
                    for register in arg_registers {
                        args.push(load_register!(
                            self,
                            last,
                            self.namespaces,
                            bytecode,
                            i,
                            register.value
                        ));
                    }
                    if callable.call.is_none() {
                        let pos = bytecode.positions.get(i).expect("Instruction out of range");
                        let exc = methodnotdefinedexc_from_str(
                            self.vm.clone(),
                            &format!(
                                "Method 'call' is not defined for '{}' type",
                                callable.typename
                            ),
                            pos.0,
                            pos.1,
                        );
                        self.raise_exc(exc);
                    }

                    let value = (callable.call.expect("Method is not defined"))(
                        callable,
                        listobject::list_from(self.vm.clone(), args),
                    );
                    maybe_handle_exception!(self, value, bytecode, i);
                    store_register!(last, self.namespaces, *result, value.unwrap());
                }

                //Control flow
                CompilerInstruction::Return { register } => {
                    let last = self.frames.last_mut().expect("No frames");
                    let res = load_register!(self, last, self.namespaces, bytecode, i, *register);
                    pop_frame!(self);
                    return res;
                }

                //Data structures
                CompilerInstruction::BuildList {
                    result,
                    value_registers,
                } => {
                    let last = self.frames.last_mut().expect("No frames");
                    let mut values = Vec::new();
                    for register in value_registers {
                        values.push(load_register!(
                            self,
                            last,
                            self.namespaces,
                            bytecode,
                            i,
                            register.value
                        ));
                    }
                    let list = listobject::list_from(self.vm.clone(), values);
                    store_register!(last, self.namespaces, *result, list);
                }
                CompilerInstruction::BuildDict {
                    result,
                    key_registers,
                    value_registers,
                } => {
                    let last = self.frames.last_mut().expect("No frames");
                    let mut map = mhash::HashMap::new();
                    for (key, value) in std::iter::zip(key_registers, value_registers) {
                        let key =
                            load_register!(self, last, self.namespaces, bytecode, i, key.value);
                        let value =
                            load_register!(self, last, self.namespaces, bytecode, i, value.value);

                        let res = map.insert(key, value);
                        maybe_handle_exception!(self, res, bytecode, i);
                    }
                    let dict = dictobject::dict_from(self.vm.clone(), map);
                    store_register!(last, self.namespaces, *result, dict);
                }
            }
        }

        pop_frame!(self);

        none_from!(self.vm)
    }
}
