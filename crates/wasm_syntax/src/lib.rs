use wasm_syntax_gen::grammar as wasm_grammar;

#[cfg(test)]
mod test;

pub trait Encode {
    fn encode(&self, buffer: &mut Vec<u8>);
}

pub trait Decode: std::marker::Sized {
    fn decode(buffer: &'_ [u8]) -> DecodeResult<'_, Self>;
}

pub type DecodeResult<'a, T> = Result<(T, &'a [u8]), DecodeError>;

#[derive(Debug, PartialEq)]
pub enum DecodeError {
    Error,
}

#[derive(Debug, PartialEq)]
pub struct Sized<T>(pub T);

#[derive(Debug, PartialEq)]
pub struct Name(pub String);

/// Zero or more `T`s. Unlive `Vec`, encoding of this type does not have a length prefix.
#[derive(Debug, PartialEq)]
pub struct Repeated<T>(pub Vec<T>);

impl Encode for u8 {
    fn encode(&self, buffer: &mut Vec<u8>) {
        buffer.push(*self);
    }
}

/// Unsigned LEB128
impl Encode for u32 {
    fn encode(&self, buffer: &mut Vec<u8>) {
        let mut val = *self;

        loop {
            let mut byte = (val & 0b0111_1111) as u8;
            val >>= 7;

            if val != 0 {
                // More bytes to come, set the continuation bit
                byte |= 0b1000_0000;
            }

            buffer.push(byte);

            if val == 0 {
                break;
            }
        }
    }
}

/// Signed LEB128
impl Encode for i32 {
    fn encode(&self, buffer: &mut Vec<u8>) {
        let mut value = *self;
        let mut more = true;

        while more {
            let mut byte = (value & 0b0111_1111) as u8;
            value >>= 7;

            if (value == 0 && byte & 0b0100_0000 == 0) || (value == -1 && byte & 0b0100_0000 != 0) {
                more = false;
            } else {
                byte |= 0b1000_0000;
            }

            buffer.push(byte);
        }
    }
}

/// Unsigned LEB128
impl Encode for u64 {
    fn encode(&self, buffer: &mut Vec<u8>) {
        let mut val = *self;

        loop {
            let mut byte = (val & 0b0111_1111) as u8;
            val >>= 7;

            if val != 0 {
                // More bytes to come, set the continuation bit
                byte |= 0b1000_0000;
            }

            buffer.push(byte);

            if val == 0 {
                break;
            }
        }
    }
}

/// Signed LEB128
impl Encode for i64 {
    fn encode(&self, buffer: &mut Vec<u8>) {
        let mut value = *self;
        let mut more = true;

        while more {
            let mut byte = (value & 0b0111_1111) as u8;
            value >>= 7;

            if (value == 0 && byte & 0b0100_0000 == 0) || (value == -1 && byte & 0b0100_0000 != 0) {
                more = false;
            } else {
                byte |= 0b1000_0000;
            }

            buffer.push(byte);
        }
    }
}

impl Encode for f32 {
    fn encode(&self, buffer: &mut Vec<u8>) {
        buffer.extend_from_slice(&self.to_le_bytes());
    }
}

impl Encode for f64 {
    fn encode(&self, buffer: &mut Vec<u8>) {
        buffer.extend_from_slice(&self.to_le_bytes());
    }
}

impl<T: Encode> Encode for Vec<T> {
    fn encode(&self, buffer: &mut Vec<u8>) {
        u32::try_from(self.len()).unwrap().encode(buffer);
        for a in self {
            a.encode(buffer);
        }
    }
}

impl<T: Encode> Encode for Repeated<T> {
    fn encode(&self, buffer: &mut Vec<u8>) {
        for a in &self.0 {
            a.encode(buffer);
        }
    }
}

impl<T: Encode> Encode for Sized<T> {
    fn encode(&self, buffer: &mut Vec<u8>) {
        let mut sized_buffer: Vec<u8> = Vec::new();
        self.0.encode(&mut sized_buffer);
        let size = u32::try_from(sized_buffer.len()).unwrap();
        size.encode(buffer);
        buffer.extend_from_slice(&sized_buffer);
    }
}

impl Encode for Name {
    fn encode(&self, buffer: &mut Vec<u8>) {
        let length = u32::try_from(self.0.len()).unwrap();
        length.encode(buffer);
        buffer.extend_from_slice(self.0.as_bytes());
    }
}

impl Decode for u8 {
    fn decode(buffer: &[u8]) -> DecodeResult<Self> {
        match buffer.first() {
            Some(byte) => Ok((*byte, &buffer[1..])),
            None => Err(DecodeError::Error),
        }
    }
}

/// Unsigned LEB128
impl Decode for u32 {
    fn decode(mut buffer: &[u8]) -> DecodeResult<Self> {
        let mut result: u32 = 0;
        let mut shift: u32 = 0;

        loop {
            let byte = match buffer.first() {
                Some(byte) => *byte,
                None => return Err(DecodeError::Error),
            };

            buffer = &buffer[1..];

            if shift == 31 && byte != 0x00 && byte != 0x01 {
                return Err(DecodeError::Error);
            }

            let low_bits = u32::from(byte & 0b0111_1111);
            result |= low_bits << shift;

            if byte & 0b1000_0000 == 0 {
                return Ok((result, buffer));
            }

            shift += 7;
        }
    }
}

/// Signed LEB128
impl Decode for i32 {
    fn decode(mut buffer: &[u8]) -> DecodeResult<Self> {
        let mut result: u32 = 0;
        let mut shift: u32 = 0;

        let mut byte;

        loop {
            byte = match buffer.first() {
                Some(byte) => *byte,
                None => return Err(DecodeError::Error),
            };

            buffer = &buffer[1..];

            result |= ((byte & 0b0111_1111) as u32) << shift;

            shift += 7;

            if byte & 0b1000_0000 == 0 {
                break;
            }
        }

        if shift < 32 && byte & 0b0100_0000 != 0 {
            result |= !0 << shift;
        }

        Ok((result as i32, buffer))
    }
}

/// Unsigned LEB128
impl Decode for u64 {
    fn decode(mut buffer: &[u8]) -> DecodeResult<Self> {
        let mut result: u64 = 0;
        let mut shift: u32 = 0;

        loop {
            let byte = match buffer.first() {
                Some(byte) => *byte,
                None => return Err(DecodeError::Error),
            };

            buffer = &buffer[1..];

            if shift == 63 && byte != 0x00 && byte != 0x01 {
                return Err(DecodeError::Error);
            }

            let low_bits = u64::from(byte & 0b0111_1111);
            result |= low_bits << shift;

            if byte & 0b1000_0000 == 0 {
                return Ok((result, buffer));
            }

            shift += 7;
        }
    }
}

/// Unsigned LEB128
impl Decode for i64 {
    fn decode(mut buffer: &[u8]) -> DecodeResult<Self> {
        let mut result: u64 = 0;
        let mut shift: u32 = 0;

        let mut byte;

        loop {
            byte = match buffer.first() {
                Some(byte) => *byte,
                None => return Err(DecodeError::Error),
            };

            buffer = &buffer[1..];

            result |= ((byte & 0b0111_1111) as u64) << shift;

            shift += 7;

            if byte & 0b1000_0000 == 0 {
                break;
            }
        }

        if shift < 64 && byte & 0b0100_0000 != 0 {
            result |= !0 << shift;
        }

        Ok((result as i64, buffer))
    }
}

impl Decode for f32 {
    fn decode(buffer: &[u8]) -> DecodeResult<Self> {
        // TODO: Check bounds
        let b1 = buffer[0];
        let b2 = buffer[1];
        let b3 = buffer[2];
        let b4 = buffer[3];
        Ok((f32::from_le_bytes([b1, b2, b3, b4]), &buffer[4..]))
    }
}

impl Decode for f64 {
    fn decode(buffer: &[u8]) -> DecodeResult<Self> {
        // TODO: Check bounds
        let b1 = buffer[0];
        let b2 = buffer[1];
        let b3 = buffer[2];
        let b4 = buffer[3];
        let b5 = buffer[4];
        let b6 = buffer[5];
        let b7 = buffer[6];
        let b8 = buffer[7];
        Ok((
            f64::from_le_bytes([b1, b2, b3, b4, b5, b6, b7, b8]),
            &buffer[8..],
        ))
    }
}

impl<T: Decode> Decode for Vec<T> {
    fn decode(mut buffer: &[u8]) -> DecodeResult<Self> {
        let (length, buffer_) = u32::decode(buffer)?;
        buffer = buffer_;

        let mut vec: Vec<T> = Vec::with_capacity(length as usize);

        for _ in 0..length {
            let (a, buffer_) = T::decode(buffer)?;
            buffer = buffer_;

            vec.push(a);
        }

        Ok((vec, buffer))
    }
}

impl<T: Decode> Decode for Repeated<T> {
    fn decode(mut buffer: &[u8]) -> DecodeResult<Self> {
        let mut stuff = Vec::new();

        while let Ok((thing, buffer_)) = T::decode(buffer) {
            buffer = buffer_;
            stuff.push(thing);
        }

        Ok((Repeated(stuff), buffer))
    }
}

impl<T: Decode> Decode for Sized<T> {
    fn decode(mut buffer: &[u8]) -> DecodeResult<Self> {
        let (size, buffer_) = u32::decode(buffer)?;
        let size = size as usize;
        buffer = buffer_;

        let sized_buffer = &buffer[..size];
        let (t, rest) = T::decode(sized_buffer)?;

        if !rest.is_empty() {
            return Err(DecodeError::Error);
        }

        Ok((Sized(t), &buffer[size..]))
    }
}

impl Decode for Name {
    fn decode(mut buffer: &[u8]) -> DecodeResult<Self> {
        let (length, buffer_) = u32::decode(buffer)?;
        let length = length as usize;
        buffer = buffer_;

        let bytes = &buffer[..length];
        let string = String::from_utf8(bytes.to_vec()).unwrap();

        Ok((Name(string), &buffer[length..]))
    }
}

wasm_grammar! {
    Module {
        0x00 0x61 0x73 0x6D // magic
        0x01 0x00 0x00 0x00 // version
        sections:repeated(Section) = Module,
    }

    Section {
        // Custom section
        0x00 custom:sized(Custom) = Custom,

        // Type section
        0x01 func_tys:sized(vec(FuncType)) = Type,

        // Import section
        0x02 imports:sized(vec(Import)) = Import,

        // Function section
        0x03 xs:sized(vec(TypeIdx)) = Function,

        // Table section
        0x04 tabs:sized(vec(Table)) = Table,

        // Memory section
        0x05 mems:sized(vec(Mem)) = Mem,

        // Global section
        0x06 globs:sized(vec(Global)) = Global,

        // Export section
        0x07 exs:sized(vec(Export)) = Export,

        // Start section
        0x08 st:sized(FuncIdx) = Start,

        // Element section
        0x09 segs:sized(vec(Elem)) = Element,

        // Code section
        0x0A codes:sized(vec(Code)) = Code,

        // Data section
        0x0B segs:sized(vec(Data)) = Data,

        // Data count section
        0x0C n:sized(u32) = DataCount,
    }

    //
    // Custom section
    //

    Custom {
        name:name bytes:repeated(u8) = Custom,
    }

    //
    // Type section
    //

    FuncType {
        0x60 r1:ResultType r2:ResultType = FuncType,
    }

    ResultType {
        tys:vec(ValType) = ResultType,
    }

    ValType {
        // t:NumType = Num,
        // t:VecType = Vec,
        // t:RefType = Ref,

        0x7F = I32,
        0x7E = I64,
        0x7D = F32,
        0x7C = F64,
        0x7B = V128,
        0x70 = FuncRef,
        0x6F = ExternRef,
    }

    NumType {
        0x7F = I32,
        0x7E = I64,
        0x7D = F32,
        0x7C = F64,
    }

    RefType {
        0x70 = FuncRef,
        0x6F = ExternRef,
    }

    VecType {
        0x7B = VecType, // TODO: This should be VecType::V128, but that makes the type enum
    }

    //
    // Import section
    //

    Import {
        module:name import_name:name desc:ImportDesc =
            Import,
    }

    ImportDesc {
        0x00 x:TypeIdx = Func,
        0x01 tt:TableType = Table,
        0x02 mt:MemType = Mem,
        0x03 gt:GlobalType = Global,
    }

    TableType {
        et:RefType lim:Limits = TableType,
    }

    MemType {
        lim:Limits = MemType,
    }

    GlobalType {
        t:ValType m:Mut = GlobalType,
    }

    Limits {
        0x00 n:u32 = Min,
        0x01 n:u32 m:u32 = MinMax,
    }

    Mut {
        0x00 = Const,
        0x01 = Mut,
    }

    //
    // Table section
    //

    Table {
        tt:TableType = Table,
    }

    //
    // Memory section
    //

    Mem {
        mt:MemType = Mem,
    }

    //
    // Global section
    //

    Global {
        gt:GlobalType e:Expr = Global,
    }

    //
    // Export section
    //

    Export {
        nm:name d:ExportDesc = Export,
    }

    ExportDesc {
        0x00 x:FuncIdx = Func,
        0x01 x:TableIdx = Table,
        0x02 x:MemIdx = Mem,
        0x03 x:GlobalIdx = Global,
    }

    //
    // Element section
    //

    Elem {
        0x00 e:Expr y:vec(FuncIdx) = E0,
        0x01 et:ElemKind y:vec(FuncIdx) = E1,
        0x02 x:TableIdx e:Expr et:ElemKind y:vec(FuncIdx) = E2,
        0x03 et:ElemKind y:vec(FuncIdx) = E3,
        0x04 e:Expr els:vec(Expr) = E4,
        0x05 et:RefType els:vec(Expr) = E5,
        0x06 x:TableIdx e:Expr et:RefType els:vec(Expr) = E6,
        0x07 et:RefType els:vec(Expr) = E7,
    }

    ElemKind {
        0x00 = ElemKind,
    }

    //
    // Code section
    //

    Code {
        code:sized(Func) = Code,
    }

    Func {
        locals:vec(Locals) e:Expr = Func,
    }

    Locals {
        n:u32 t:ValType = Locals,
    }

    //
    // Data section
    //

    Data {
        0x00 e:Expr bytes:vec(u8) = D0,
        0x01 bytes:vec(u8) = D1,
        0x02 x:MemIdx e:Expr bytes:vec(u8) = D2,
    }

    //
    // Indices
    //

    TypeIdx {
        x:u32 = TypeIdx,
    }

    TableIdx {
        x:u32 = TableIdx,
    }

    FuncIdx {
        x:u32 = FuncIdx,
    }

    MemIdx {
        x:u32 = MemIdx,
    }

    GlobalIdx {
        x:u32 = GlobalIdx,
    }

    LabelIdx {
        x:u32 = LabelIdx,
    }

    LocalIdx {
        x:u32 = LocalIdx,
    }

    ElemIdx {
        x:u32 = ElemIdx,
    }

    DataIdx {
        x:u32 = DataIdx,
    }

    //
    // Expressions
    //

    Expr {
        instrs:repeated(Instr) 0x0B = Expr,
    }

    Instr {

        //
        // Control instructions
        //

        0x00 = Unreachable,
        0x01 = Nop,
        0x02 bt:i32 instrs:repeated(Instr) 0x0B = Block,
        0x03 bt:i32 instrs:repeated(Instr) 0x0B = Loop,
        0x04 bt:i32 instrs:repeated(Instr) else_:Else = If,
        0x0C l:LabelIdx = Br,
        0x0D l:LabelIdx = BrIf,
        0x0E ls:vec(LabelIdx) ln:LabelIdx = BrTable,
        0x0F = Return,
        0x10 x:FuncIdx = Call,
        0x11 y:TypeIdx x:TableIdx = CallIndirect,

        //
        // Reference instructions
        //

        0xD0 t:RefType = RefNull,
        0xD1 = RefIsNull,
        0xD2 x:FuncIdx = RefFunc,

        //
        // Parametric instructions
        //

        0x1A = Drop,
        0x1B = Select,
        0x1C tys:vec(ValType) = SelectTys,

        //
        // Variable instructions
        //

        0x20 x:LocalIdx = LocalGet,
        0x21 x:LocalIdx = LocalSet,
        0x22 x:LocalIdx = LocalTee,
        0x23 x:GlobalIdx = GlobalGet,
        0x24 x:GlobalIdx = GlobalSet,

        //
        // Table instructions
        //

        0x25 x:TableIdx = TableGet,
        0x26 x:TableIdx = TableSet,
        0xFC 12:u32 y:ElemIdx x:TableIdx = TableInit,
        0xFC 13:u32 x:ElemIdx = ElemDrop,
        0xFC 14:u32 x:TableIdx y:TableIdx = TableCopy,
        0xFC 15:u32 x:TableIdx = TableGrow,
        0xFC 16:u32 x:TableIdx = TableSize,
        0xFC 17:u32 x:TableIdx = TableFill,

        //
        // Memory instructions
        //

        0x28 m:MemArg = I32Load,
        0x29 m:MemArg = I64Load,
        0x2A m:MemArg = F32Load,
        0x2B m:MemArg = F64Load,
        0x2C m:MemArg = I32Load8S,
        0x2D m:MemArg = I32Load8U,
        0x2E m:MemArg = I32Load16S,
        0x2F m:MemArg = I32Load16U,
        0x30 m:MemArg = I64Load8S,
        0x31 m:MemArg = I64Load8U,
        0x32 m:MemArg = I64Load16S,
        0x33 m:MemArg = I64Load16U,
        0x34 m:MemArg = I64Load32S,
        0x35 m:MemArg = I64Load32U,
        0x36 m:MemArg = I32Store,
        0x37 m:MemArg = I64Store,
        0x38 m:MemArg = F32Store,
        0x39 m:MemArg = F64Store,
        0x3A m:MemArg = I32Store8,
        0x3B m:MemArg = I32Store16,
        0x3C m:MemArg = I64Store8,
        0x3D m:MemArg = I64Store16,
        0x3E m:MemArg = I64Store32,
        0x3F 0u32 = MemorySize,
        0x40 0u32 = MemoryGrow,
        0xFC 8:u32 x:DataIdx 0u32 = MemoryInit,
        0xFC 9:u32 x:DataIdx = DataDrop,
        0xFC 10:u32 0u32 0u32 = MemoryCopy,
        0xFC 11:u32 0u32 = MemoryFill,

        //
        // Numeric instructions
        //

        0x41 n:i32 = I32Const,
        0x42 n:i64 = I64Const,
        0x43 z:f32 = F32Const,
        0x44 z:f64 = F64Const,

        0x45 = I32Eqz,
        0x46 = I32Eq,
        0x47 = I32Ne,
        0x48 = I32LtS,
        0x49 = I32LtU,
        0x4A = I32GtS,
        0x4B = I32GtU,
        0x4C = I32LeS,
        0x4D = I32LeU,
        0x4E = I32GeS,
        0x4F = I32GeU,

        0x50 = I64EqZ,
        0x51 = I64Eq,
        0x52 = I64Ne,
        0x53 = I64LtS,
        0x54 = I64LtU,
        0x55 = I64GtS,
        0x56 = I64GtU,
        0x57 = I64LeS,
        0x58 = I64LeU,
        0x59 = I64GeS,
        0x5A = I64GeU,

        0x5B = F32Eq,
        0x5C = F32Ne,
        0x5D = F32Lt,
        0x5E = F32Gt,
        0x5F = F32Le,
        0x60 = F32Ge,

        0x61 = F64Eq,
        0x62 = F64Ne,
        0x63 = F64Lt,
        0x64 = F64Gt,
        0x65 = F64Le,
        0x66 = F64Ge,

        0x67 = I32Clz,
        0x68 = I32Ctz,
        0x69 = I32Popcnt,
        0x6A = I32Add,
        0x6B = I32Sub,
        0x6C = I32Mul,
        0x6D = I32DivS,
        0x6E = I32DivU,
        0x6F = I32RemS,
        0x70 = I32RemU,
        0x71 = I32And,
        0x72 = I32Or,
        0x73 = I32Xor,
        0x74 = I32Shl,
        0x75 = I32ShrS,
        0x76 = I32ShrU,
        0x77 = I32Rotl,
        0x78 = I32Rotr,

        0x79 = I64Clz,
        0x7A = I64Ctz,
        0x7B = I64Popcnt,
        0x7C = I64Add,
        0x7D = I64Sub,
        0x7E = I64Mul,
        0x7F = I64DivS,
        0x80 = I64DivU,
        0x81 = I64RemS,
        0x82 = I64RemU,
        0x83 = I64And,
        0x84 = I64Or,
        0x85 = I64Xor,
        0x86 = I64Shl,
        0x87 = I64ShrS,
        0x88 = I64ShrU,
        0x89 = I64Rotl,
        0x8A = I64Rotr,

        0x8B = F32Abs,
        0x8C = F32Neg,
        0x8D = F32Ceil,
        0x8E = F32Floor,
        0x8F = F32Trunc,
        0x90 = F32Nearest,
        0x91 = F32Sqrt,
        0x92 = F32Add,
        0x93 = F32Sub,
        0x94 = F32Mul,
        0x95 = F32Div,
        0x96 = F32Min,
        0x97 = F32Max,
        0x98 = F32Copysign,

        0x99 = F64Abs,
        0x9A = F64Neg,
        0x9B = F64Ceil,
        0x9C = F64Floor,
        0x9D = F64Trunc,
        0x9E = F64Nearest,
        0x9F = F64Sqrt,
        0xA0 = F64Add,
        0xA1 = F64Sub,
        0xA2 = F64Mul,
        0xA3 = F64Div,
        0xA4 = F64Min,
        0xA5 = F64Max,
        0xA6 = F64Copysign,

        0xA7 = I32WrapI64,
        0xA8 = I32TruncF32S,
        0xA9 = I32TruncF32U,
        0xAA = I32TruncF64S,
        0xAB = I32TruncF64U,
        0xAC = I64ExtendI32S,
        0xAD = I64ExtendI32U,
        0xAE = I64TruncF32S,
        0xAF = I64TruncF32U,
        0xB0 = I64TruncF64S,
        0xB1 = I64TruncF64U,
        0xB2 = F32ConvertI32S,
        0xB3 = F32ConvertI32U,
        0xB4 = F32ConvertI64S,
        0xB5 = F32ConvertI64U,
        0xB6 = F32DemoteF64,
        0xB7 = F64ConvertI32S,
        0xB8 = F64ConvertI32U,
        0xB9 = F64ConvertI64S,
        0xBA = F64ConvertI64U,
        0xBB = F64PromoteF32,
        0xBC = I32ReinterpretF32,
        0xBD = I64ReinterpretF64,
        0xBE = F32ReinterpretI32,
        0xBF = F64ReinterpretI64,

        0xC0 = I32Extend8S,
        0xC1 = I32Extend16S,
        0xC2 = I64Extend8S,
        0xC3 = I64Extend16S,
        0xC4 = I64Extend32S,

        0xFC 0:u32 = I32TruncSatF32S,
        0xFC 1:u32 = I32TruncSatF32U,
        0xFC 2:u32 = I32TruncSatF64S,
        0xFC 3:u32 = I32TruncSatF64U,
        0xFC 4:u32 = I64TruncSatF32S,
        0xFC 5:u32 = I64TruncSatF32U,
        0xFC 6:u32 = I64TruncSatF64S,
        0xFC 7:u32 = I64TruncSatF64U,

        // TODO: Vector instructions
    }

    Else {
        0x0B = NoElse,
        0x05 instrs:repeated(Instr) 0x0B = Else,
    }

    MemArg {
        align:u32 offset:u32 = MemArg,
    }
}
