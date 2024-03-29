// Interpret bytecode

use crate::objects::exceptionobject::{self, methodnotdefinedexc_from_str};
use crate::objects::{
    classtype, dictobject, mhash, noneobject, stringobject, RawObject, TypeObject,
};
use crate::parser::Position;
use crate::{
    compiler::{Bytecode, CompilerInstruction, CompilerRegister},
    fileinfo::FileInfo,
    none_from,
    objects::{boolobject, fnobject, intobject, listobject, Object},
    stats, TimeitHolder,
};
use colored::Colorize;
use std::marker::PhantomData;
use std::ops::DerefMut;
use std::time::Instant;
use trc::Trc;

#[derive(Clone, PartialEq, Eq)]
pub struct Namespaces<'a> {
    variables: Vec<Vec<Option<Object<'a>>>>,
    _marker: PhantomData<&'a ()>,
}

pub const MIN_INT_CACHE: isize = -5;
pub const MAX_INT_CACHE: isize = 256;
pub const INT_CACHE_SIZE: isize = MAX_INT_CACHE - MIN_INT_CACHE;
pub const INT_CACHE_OFFSET: isize = MIN_INT_CACHE.abs();

#[derive(Clone)]
pub struct SingletonCache<'a> {
    pub int_cache: [Option<Object<'a>>; INT_CACHE_SIZE as usize],
    pub bool_cache: (Option<Object<'a>>, Option<Object<'a>>),
    pub none_singleton: Option<Object<'a>>,
    _marker: PhantomData<&'a ()>,
}

#[derive(Clone)]
pub struct Types<'a> {
    pub typetp: Option<Trc<TypeObject<'a>>>,
    pub objecttp: Option<Trc<TypeObject<'a>>>,
    pub inttp: Option<Trc<TypeObject<'a>>>,
    pub booltp: Option<Trc<TypeObject<'a>>>,
    pub codetp: Option<Trc<TypeObject<'a>>>,
    pub dicttp: Option<Trc<TypeObject<'a>>>,
    pub exctp: Option<Trc<TypeObject<'a>>>,
    pub nameexctp: Option<Trc<TypeObject<'a>>>,
    pub overflwexctp: Option<Trc<TypeObject<'a>>>,
    pub mthntfndexctp: Option<Trc<TypeObject<'a>>>,
    pub tpmisexctp: Option<Trc<TypeObject<'a>>>,
    pub keyntfndexctp: Option<Trc<TypeObject<'a>>>,
    pub valueexctp: Option<Trc<TypeObject<'a>>>,
    pub divzeroexctp: Option<Trc<TypeObject<'a>>>,
    pub fntp: Option<Trc<TypeObject<'a>>>,
    pub listtp: Option<Trc<TypeObject<'a>>>,
    pub nonetp: Option<Trc<TypeObject<'a>>>,
    pub strtp: Option<Trc<TypeObject<'a>>>,
    pub classtp: Option<Trc<TypeObject<'a>>>,
    pub attrexctp: Option<Trc<TypeObject<'a>>>,
    pub methodtp: Option<Trc<TypeObject<'a>>>,

    pub n_types: u32,
}

#[derive(Clone)]
pub struct VM<'a> {
    pub types: Trc<Types<'a>>,
    pub interpreters: Vec<Trc<Interpreter<'a>>>,
    pub namespaces: Trc<Namespaces<'a>>,
    info: FileInfo<'a>,
    pub cache: SingletonCache<'a>,
}

impl<'a> Eq for VM<'a> {}

impl<'a> PartialEq for VM<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.namespaces == other.namespaces
    }
}

#[derive(Clone)]
pub struct Interpreter<'a> {
    frames: Vec<Frame<'a>>,
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

#[macro_export]
macro_rules! unwrap_fast {
    ($expr:expr) => {
        unsafe { $expr.unwrap_unchecked() }
    };
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
            types: Trc::new(Types {
                typetp: None,
                objecttp: None,
                inttp: None,
                booltp: None,
                codetp: None,
                dicttp: None,
                exctp: None,
                nameexctp: None,
                overflwexctp: None,
                mthntfndexctp: None,
                tpmisexctp: None,
                keyntfndexctp: None,
                valueexctp: None,
                divzeroexctp: None,
                fntp: None,
                listtp: None,
                nonetp: None,
                strtp: None,
                classtp: None,
                attrexctp: None,
                methodtp: None,
                n_types: 0,
            }),
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
        let int_cache_arr_ref = &this.cache.int_cache;
        let ptr = int_cache_arr_ref as *const [Option<Object>; INT_CACHE_SIZE as usize]
            as *mut [Option<Object>; INT_CACHE_SIZE as usize];
        intobject::generate_cache(
            this.clone(),
            this.types.inttp.as_ref().unwrap().clone(),
            ptr,
        );

        let bool_cache_tup_ref = &this.cache.bool_cache;
        let ptr = bool_cache_tup_ref as *const (Option<Object>, Option<Object>)
            as *mut (Option<Object>, Option<Object>);
        boolobject::generate_cache(
            this.clone(),
            this.types.booltp.as_ref().unwrap().clone(),
            ptr,
        );

        let none_obj_ref = &this.cache.none_singleton;
        let ptr = none_obj_ref as *const Option<Object> as *mut Option<Object>;
        noneobject::generate_cache(
            this.clone(),
            this.types.nonetp.as_ref().unwrap().clone(),
            ptr,
        );
    }

    pub fn execute(mut this: Trc<Self>, bytecode: &Bytecode<'a>) -> Object<'a> {
        let interpreter = Interpreter::new(this.namespaces.clone(), this.clone());

        this.interpreters.push(Trc::new(interpreter));
        let last = unwrap_fast!(this.deref_mut().interpreters.last_mut());
        return last.run_interpreter(bytecode);
    }

    pub fn execute_timeit(
        mut this: Trc<Self>,
        bytecode: &Bytecode<'a>,
        timeit: &mut TimeitHolder,
    ) -> Object<'a> {
        //See bench.rs, this is a very similar implementation (pub fn iter<T, F>(inner: &mut F) -> stats::Summary)

        let samples = &mut [0f64; 50];

        //Get initial result
        let mut res =
            (unwrap_fast!(this.deref_mut().interpreters.last_mut())).run_interpreter(bytecode);

        for p in &mut *samples {
            let mut time = 0;
            let mut i = 0;
            while time == 0 && i < 10 {
                let last = unwrap_fast!(this.deref_mut().interpreters.last_mut());
                let start = Instant::now();
                for _ in 0..5 {
                    res = last.run_interpreter(bytecode);
                }
                let delta = start.elapsed().as_nanos();
                time = if (delta as i128 / 5_i128) - (timeit.baseline as i128) < 0 {
                    0
                } else {
                    delta / 5 - timeit.baseline
                };
                i += 1;
            }
            if time > 0 {
                *p = time as f64;
            } else {
                *p = 0.001;
            }
        }

        stats::winsorize(samples, 5.0);

        let sum: f64 = samples.iter().sum();

        timeit.time = sum / samples.len() as f64;

        res
    }

    pub fn execute_vars(
        mut this: Trc<Self>,
        bytecode: &Bytecode<'a>,
        vars: hashbrown::HashMap<isize, Object<'a>>,
    ) -> Object<'a> {
        let interpreter = Interpreter::new(this.namespaces.clone(), this.clone());
        this.interpreters.push(Trc::new(interpreter));

        let res = (unwrap_fast!(this.deref_mut().interpreters.last_mut()))
            .run_interpreter_vars(bytecode, vars);
        this.interpreters.pop();
        res
    }

    pub fn execute_extract_namespace(
        mut this: Trc<Self>,
        bytecode: &Bytecode<'a>,
    ) -> Vec<Option<Trc<RawObject<'a>>>> {
        let interpreter = Interpreter::new(this.namespaces.clone(), this.clone());
        this.interpreters.push(Trc::new(interpreter));

        let res = (unwrap_fast!(this.deref_mut().interpreters.last_mut()))
            .run_interpreter_extract_namespace(bytecode);
        this.interpreters.pop();
        res
    }

    pub fn terminate(_: Trc<Self>) -> ! {
        //Clean up child threads here
        std::process::exit(1);
    }
}

macro_rules! load_register {
    ($this:expr, $last:expr, $last_vars:expr, $bytecode:expr, $i:expr, $register:expr) => {
        match $register {
            CompilerRegister::R(v) => $last.registers[v].clone(),
            CompilerRegister::V(v) => match &$last_vars[v] {
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
            CompilerRegister::C(v) => unwrap_fast!($bytecode.consts.get(v)).clone(),
        }
    };
}

macro_rules! store_register {
    ($last:expr, $last_vars:expr, $register:expr, $value:expr) => {
        match $register {
            CompilerRegister::R(v) => $last.registers[v] = $value,
            CompilerRegister::V(v) => $last_vars[v] = Some($value),
            CompilerRegister::C(_) => unreachable!("Impossible."),
        }
    };
}

impl<'a> Interpreter<'a> {
    pub fn new(namespaces: Trc<Namespaces<'a>>, vm: Trc<VM<'a>>) -> Interpreter<'a> {
        Interpreter {
            frames: Vec::new(),
            namespaces,
            vm,
        }
    }

    #[allow(dead_code)]
    pub fn raise_exc(&self, exc_obj: Object<'a>) -> ! {
        let exc = unsafe { &exc_obj.internals.exc }.clone();
        self.raise_exc_pos(exc_obj, exc.start, exc.end);
    }

    fn raise_exc_pos(&self, exc_obj: Object<'a>, start: Position, end: Position) -> ! {
        let header: String = match RawObject::object_repr_safe(exc_obj) {
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
        bytecode: &Bytecode<'a>,
        vars: hashbrown::HashMap<isize, Object<'a>>,
    ) -> Object<'a> {
        add_frame!(
            self,
            bytecode.n_registers as usize,
            bytecode.n_variables as usize
        );

        for (i, var) in unwrap_fast!(self.namespaces.variables.last_mut())
            .iter_mut()
            .enumerate()
        {
            if vars.get(&(i as isize)).is_some() {
                *var = Some(unwrap_fast!(vars.get(&(i as isize))).clone());
            }
        }

        let res = self.run_interpreter_raw(bytecode);
        pop_frame!(self);
        res
    }

    pub fn run_interpreter(&mut self, bytecode: &Bytecode<'a>) -> Object<'a> {
        if !bytecode.instructions.is_empty() {
            add_frame!(
                self,
                bytecode.n_registers as usize,
                bytecode.n_variables as usize
            );
            let res = self.run_interpreter_raw(bytecode);
            pop_frame!(self);
            return res;
        }
        none_from!(self.vm)
    }

    pub fn run_interpreter_extract_namespace(
        &mut self,
        bytecode: &Bytecode<'a>,
    ) -> Vec<Option<Trc<RawObject<'a>>>> {
        add_frame!(
            self,
            bytecode.n_registers as usize,
            bytecode.n_variables as usize
        );

        if !bytecode.instructions.is_empty() {
            self.run_interpreter_raw(bytecode);
        }

        let last = self.namespaces.variables.last().unwrap().clone();
        pop_frame!(self);
        last
    }

    #[inline]
    pub fn run_interpreter_raw(&mut self, bytecode: &Bytecode<'a>) -> Object<'a> {
        let last = unwrap_fast!(self.frames.last_mut());
        let last_vars = unwrap_fast!(self.namespaces.variables.last_mut());
        for instruction in bytecode.instructions.iter() {
            match instruction {
                //Binary operations
                CompilerInstruction::BinaryAdd { a, b, result, i } => {
                    let selfv = load_register!(self, last, last_vars, bytecode, *i, *a);
                    if selfv.tp.add.is_none() {
                        let pos = bytecode
                            .positions
                            .get(*i)
                            .expect("Instruction out of range");
                        let exc = methodnotdefinedexc_from_str(
                            self.vm.clone(),
                            &format!(
                                "Method 'add' is not defined for '{}' type",
                                selfv.tp.typename
                            ),
                            pos.0,
                            pos.1,
                        );
                        self.raise_exc(exc);
                    }
                    let res = unwrap_fast!(selfv.tp.add)(
                        selfv,
                        load_register!(self, last, last_vars, bytecode, *i, *b),
                    );
                    maybe_handle_exception!(self, res, bytecode, *i);
                    store_register!(last, last_vars, *result, unwrap_fast!(res));
                }
                CompilerInstruction::BinarySub { a, b, result, i } => {
                    let selfv = load_register!(self, last, last_vars, bytecode, *i, *a);
                    if selfv.tp.sub.is_none() {
                        let pos = bytecode
                            .positions
                            .get(*i)
                            .expect("Instruction out of range");
                        let exc = methodnotdefinedexc_from_str(
                            self.vm.clone(),
                            &format!(
                                "Method 'sub' is not defined for '{}' type",
                                selfv.tp.typename
                            ),
                            pos.0,
                            pos.1,
                        );
                        self.raise_exc(exc);
                    }
                    let res = unwrap_fast!(selfv.tp.sub)(
                        selfv,
                        load_register!(self, last, last_vars, bytecode, *i, *b),
                    );
                    maybe_handle_exception!(self, res, bytecode, *i);
                    store_register!(last, last_vars, *result, unwrap_fast!(res));
                }
                CompilerInstruction::BinaryMul { a, b, result, i } => {
                    let selfv = load_register!(self, last, last_vars, bytecode, *i, *a);
                    if selfv.tp.mul.is_none() {
                        let pos = bytecode
                            .positions
                            .get(*i)
                            .expect("Instruction out of range");
                        let exc = methodnotdefinedexc_from_str(
                            self.vm.clone(),
                            &format!(
                                "Method 'mul' is not defined for '{}' type",
                                selfv.tp.typename
                            ),
                            pos.0,
                            pos.1,
                        );
                        self.raise_exc(exc);
                    }
                    let res = unwrap_fast!(selfv.tp.mul)(
                        selfv,
                        load_register!(self, last, last_vars, bytecode, *i, *b),
                    );
                    maybe_handle_exception!(self, res, bytecode, *i);
                    store_register!(last, last_vars, *result, unwrap_fast!(res));
                }
                CompilerInstruction::BinaryDiv { a, b, result, i } => {
                    let selfv = load_register!(self, last, last_vars, bytecode, *i, *a);
                    if selfv.tp.div.is_none() {
                        let pos = bytecode
                            .positions
                            .get(*i)
                            .expect("Instruction out of range");
                        let exc = methodnotdefinedexc_from_str(
                            self.vm.clone(),
                            &format!(
                                "Method 'div' is not defined for '{}' type",
                                selfv.tp.typename
                            ),
                            pos.0,
                            pos.1,
                        );
                        self.raise_exc(exc);
                    }
                    let res = unwrap_fast!(selfv.tp.div)(
                        selfv,
                        load_register!(self, last, last_vars, bytecode, *i, *b),
                    );
                    maybe_handle_exception!(self, res, bytecode, *i);
                    store_register!(last, last_vars, *result, unwrap_fast!(res));
                }

                //Unary operations
                CompilerInstruction::UnaryNeg { a, result, i } => {
                    let selfv = load_register!(self, last, last_vars, bytecode, *i, *a);
                    if selfv.tp.neg.is_none() {
                        let pos = bytecode
                            .positions
                            .get(*i)
                            .expect("Instruction out of range");
                        let exc = methodnotdefinedexc_from_str(
                            self.vm.clone(),
                            &format!(
                                "Method 'neg' is not defined for '{}' type",
                                selfv.tp.typename
                            ),
                            pos.0,
                            pos.1,
                        );
                        self.raise_exc(exc);
                    }
                    let res = unwrap_fast!(selfv.tp.neg)(selfv);
                    maybe_handle_exception!(self, res, bytecode, *i);
                    store_register!(last, last_vars, *result, unwrap_fast!(res));
                }

                //Register manipulation
                CompilerInstruction::CopyRegister { from, to, i } => {
                    store_register!(
                        last,
                        last_vars,
                        *to,
                        load_register!(self, last, last_vars, bytecode, *i, *from)
                    );
                }
                CompilerInstruction::AttrLoad {
                    left,
                    attridx,
                    result,
                    i,
                } => {
                    let attr = load_register!(self, last, last_vars, bytecode, *i, *attridx);
                    let selfv = load_register!(self, last, last_vars, bytecode, *i, *left);

                    if selfv.tp.getattr.is_none() {
                        let pos = bytecode
                            .positions
                            .get(*i)
                            .expect("Instruction out of range");
                        let exc = methodnotdefinedexc_from_str(
                            self.vm.clone(),
                            &format!(
                                "Method 'getattr' is not defined for '{}' type",
                                selfv.tp.typename
                            ),
                            pos.0,
                            pos.1,
                        );
                        self.raise_exc(exc);
                    }

                    let res = unwrap_fast!(selfv.tp.getattr)(selfv, attr);
                    maybe_handle_exception!(self, res, bytecode, *i);
                    store_register!(last, last_vars, *result, unwrap_fast!(res));
                }

                //Functions, arguments
                CompilerInstruction::MakeFunction {
                    nameidx,
                    argsidx,
                    codeidx,
                    out,
                } => {
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
                    let func = fnobject::fn_from(
                        self.vm.clone(),
                        code,
                        unsafe { &args.internals.arr }.to_vec(),
                        unsafe { &name.internals.str }.to_string(),
                    );
                    store_register!(last, last_vars, *out, func);
                }
                CompilerInstruction::Call {
                    callableregister,
                    result,
                    arg_registers,
                    i,
                } => {
                    let callable =
                        load_register!(self, last, last_vars, bytecode, *i, *callableregister);
                    let mut args = Vec::new();
                    for register in arg_registers {
                        args.push(load_register!(
                            self,
                            last,
                            last_vars,
                            bytecode,
                            *i,
                            register.value
                        ));
                    }
                    if callable.tp.call.is_none() {
                        let pos = bytecode
                            .positions
                            .get(*i)
                            .expect("Instruction out of range");
                        let exc = methodnotdefinedexc_from_str(
                            self.vm.clone(),
                            &format!(
                                "Method 'call' is not defined for '{}' type",
                                callable.tp.typename
                            ),
                            pos.0,
                            pos.1,
                        );
                        self.raise_exc(exc);
                    }

                    let value = (callable.tp.call.expect("Method is not defined"))(
                        callable,
                        listobject::list_from(self.vm.clone(), args),
                    );
                    maybe_handle_exception!(self, value, bytecode, *i);
                    store_register!(last, last_vars, *result, unwrap_fast!(value));
                }

                //Control flow
                CompilerInstruction::Return { register, i } => {
                    let res = load_register!(self, last, last_vars, bytecode, *i, *register);
                    pop_frame!(self);
                    return res;
                }

                //Data structures
                CompilerInstruction::BuildList {
                    result,
                    value_registers,
                    i,
                } => {
                    let mut values = Vec::new();
                    for register in value_registers {
                        values.push(load_register!(
                            self, last, last_vars, bytecode, *i, *register
                        ));
                    }
                    let list = listobject::list_from(self.vm.clone(), values);
                    store_register!(last, last_vars, *result, list);
                }
                CompilerInstruction::BuildDict {
                    result,
                    key_registers,
                    value_registers,
                    i,
                } => {
                    let mut map = mhash::HashMap::new();
                    for (key, value) in std::iter::zip(key_registers, value_registers) {
                        let key = load_register!(self, last, last_vars, bytecode, *i, *key);
                        let value = load_register!(self, last, last_vars, bytecode, *i, *value);

                        let res = map.insert(key, value);
                        maybe_handle_exception!(self, res, bytecode, *i);
                    }
                    let dict = dictobject::dict_from(self.vm.clone(), map);
                    store_register!(last, last_vars, *result, dict);
                }

                //Class
                CompilerInstruction::MakeClass {
                    name,
                    methods,
                    bytecode: class_body,
                    out,
                } => {
                    let mut method_map = mhash::HashMap::new();

                    let namespace =
                        VM::<'a>::execute_extract_namespace(self.vm.clone(), class_body);
                    for i in 0..namespace.len() {
                        let var = namespace.get(i).unwrap();
                        debug_assert!(var.is_some());
                        method_map.insert(
                            stringobject::string_from(
                                self.vm.clone(),
                                methods.get(&(i as i32)).unwrap().to_string(),
                            ),
                            var.as_ref().unwrap().clone(),
                        );
                    }

                    let method_dict = dictobject::dict_from(self.vm.clone(), method_map);

                    let new_class =
                        classtype::create_class(self.vm.clone(), name.clone(), method_dict);

                    store_register!(last, last_vars, *out, new_class);
                }
            }
        }

        none_from!(self.vm)
    }
}
