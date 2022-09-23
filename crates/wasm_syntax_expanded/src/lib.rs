pub trait Encode {
    fn encode(&self, buffer: &mut Vec<u8>);
}
pub trait Decode: std::marker::Sized {
    fn decode(buffer: &'_ [u8]) -> DecodeResult<'_, Self>;
}
pub type DecodeResult<'a, T> = Result<(T, &'a [u8]), DecodeError>;
#[derive(:: core :: fmt :: Debug, :: core :: cmp :: PartialEq)]
pub enum DecodeError {
    Error,
}
#[derive(:: core :: fmt :: Debug, :: core :: cmp :: PartialEq)]
pub struct Sized<T>(pub T);
#[derive(:: core :: fmt :: Debug, :: core :: cmp :: PartialEq)]
pub struct Name(pub String);
#[doc = " Zero or more `T`s. Unlive `Vec`, encoding of this type does not have a length prefix."]
#[derive(:: core :: fmt :: Debug, :: core :: cmp :: PartialEq)]
pub struct Repeated<T>(pub Vec<T>);
impl Encode for u8 {
    fn encode(&self, buffer: &mut Vec<u8>) {
        buffer.push(*self);
    }
}
#[doc = " Unsigned LEB128"]
impl Encode for u32 {
    fn encode(&self, buffer: &mut Vec<u8>) {
        let mut val = *self;
        loop {
            let mut byte = (val & 0b0111_1111) as u8;
            val >>= 7;
            if val != 0 {
                byte |= 0b1000_0000;
            }
            buffer.push(byte);
            if val == 0 {
                break;
            }
        }
    }
}
#[doc = " Signed LEB128"]
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
#[doc = " Unsigned LEB128"]
impl Encode for u64 {
    fn encode(&self, buffer: &mut Vec<u8>) {
        let mut val = *self;
        loop {
            let mut byte = (val & 0b0111_1111) as u8;
            val >>= 7;
            if val != 0 {
                byte |= 0b1000_0000;
            }
            buffer.push(byte);
            if val == 0 {
                break;
            }
        }
    }
}
#[doc = " Signed LEB128"]
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
#[doc = " Unsigned LEB128"]
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
#[doc = " Signed LEB128"]
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
#[doc = " Unsigned LEB128"]
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
#[doc = " Unsigned LEB128"]
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
        let b1 = buffer[0];
        let b2 = buffer[1];
        let b3 = buffer[2];
        let b4 = buffer[3];
        Ok((f32::from_le_bytes([b1, b2, b3, b4]), &buffer[4..]))
    }
}
impl Decode for f64 {
    fn decode(buffer: &[u8]) -> DecodeResult<Self> {
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
#[derive(:: core :: fmt :: Debug, :: core :: cmp :: PartialEq)]
pub struct Module(Repeated<Section>);
impl Encode for Module {
    fn encode(&self, buffer: &mut Vec<u8>) {
        let Module(sections) = self;
        buffer.push(0u8);
        buffer.push(97u8);
        buffer.push(115u8);
        buffer.push(109u8);
        buffer.push(1u8);
        buffer.push(0u8);
        buffer.push(0u8);
        buffer.push(0u8);
        sections.encode(buffer);
    }
}
impl Decode for Module {
    fn decode(mut buffer: &[u8]) -> DecodeResult<Module> {
        let (lit, buffer_) = u8::decode(buffer)?;
        if lit != 0u8 {
            return Err(DecodeError::Error);
        }
        buffer = buffer_;
        let (lit, buffer_) = u8::decode(buffer)?;
        if lit != 97u8 {
            return Err(DecodeError::Error);
        }
        buffer = buffer_;
        let (lit, buffer_) = u8::decode(buffer)?;
        if lit != 115u8 {
            return Err(DecodeError::Error);
        }
        buffer = buffer_;
        let (lit, buffer_) = u8::decode(buffer)?;
        if lit != 109u8 {
            return Err(DecodeError::Error);
        }
        buffer = buffer_;
        let (lit, buffer_) = u8::decode(buffer)?;
        if lit != 1u8 {
            return Err(DecodeError::Error);
        }
        buffer = buffer_;
        let (lit, buffer_) = u8::decode(buffer)?;
        if lit != 0u8 {
            return Err(DecodeError::Error);
        }
        buffer = buffer_;
        let (lit, buffer_) = u8::decode(buffer)?;
        if lit != 0u8 {
            return Err(DecodeError::Error);
        }
        buffer = buffer_;
        let (lit, buffer_) = u8::decode(buffer)?;
        if lit != 0u8 {
            return Err(DecodeError::Error);
        }
        buffer = buffer_;
        let (sections, buffer_) = Repeated::<Section>::decode(buffer)?;
        buffer = buffer_;
        Ok((Module(sections), buffer))
    }
}
#[derive(:: core :: fmt :: Debug, :: core :: cmp :: PartialEq)]
pub enum Section {
    Custom(Sized<Custom>),
    Type(Sized<Vec<FuncType>>),
    Import(Sized<Vec<Import>>),
    Function(Sized<Vec<TypeIdx>>),
    Table(Sized<Vec<Table>>),
    Mem(Sized<Vec<Mem>>),
    Global(Sized<Vec<Global>>),
    Export(Sized<Vec<Export>>),
    Start(Sized<FuncIdx>),
    Element(Sized<Vec<Elem>>),
    Code(Sized<Vec<Code>>),
    Data(Sized<Vec<Data>>),
    DataCount(Sized<u32>),
}
impl Encode for Section {
    fn encode(&self, buffer: &mut Vec<u8>) {
        match self {
            Section::Custom(custom) => {
                0u8.encode(buffer);
                custom.encode(buffer);
            }
            Section::Type(func_tys) => {
                1u8.encode(buffer);
                func_tys.encode(buffer);
            }
            Section::Import(imports) => {
                2u8.encode(buffer);
                imports.encode(buffer);
            }
            Section::Function(xs) => {
                3u8.encode(buffer);
                xs.encode(buffer);
            }
            Section::Table(tabs) => {
                4u8.encode(buffer);
                tabs.encode(buffer);
            }
            Section::Mem(mems) => {
                5u8.encode(buffer);
                mems.encode(buffer);
            }
            Section::Global(globs) => {
                6u8.encode(buffer);
                globs.encode(buffer);
            }
            Section::Export(exs) => {
                7u8.encode(buffer);
                exs.encode(buffer);
            }
            Section::Start(st) => {
                8u8.encode(buffer);
                st.encode(buffer);
            }
            Section::Element(segs) => {
                9u8.encode(buffer);
                segs.encode(buffer);
            }
            Section::Code(codes) => {
                10u8.encode(buffer);
                codes.encode(buffer);
            }
            Section::Data(segs) => {
                11u8.encode(buffer);
                segs.encode(buffer);
            }
            Section::DataCount(n) => {
                12u8.encode(buffer);
                n.encode(buffer);
            }
        }
    }
}
impl Decode for Section {
    fn decode(mut buffer: &[u8]) -> DecodeResult<Section> {
        match buffer {
            [0u8, ..] => {
                buffer = &buffer[1usize..];
                let (custom, buffer_) = Sized::<Custom>::decode(buffer)?;
                buffer = buffer_;
                Ok((Section::Custom(custom), buffer))
            }
            [1u8, ..] => {
                buffer = &buffer[1usize..];
                let (func_tys, buffer_) = Sized::<Vec<FuncType>>::decode(buffer)?;
                buffer = buffer_;
                Ok((Section::Type(func_tys), buffer))
            }
            [2u8, ..] => {
                buffer = &buffer[1usize..];
                let (imports, buffer_) = Sized::<Vec<Import>>::decode(buffer)?;
                buffer = buffer_;
                Ok((Section::Import(imports), buffer))
            }
            [3u8, ..] => {
                buffer = &buffer[1usize..];
                let (xs, buffer_) = Sized::<Vec<TypeIdx>>::decode(buffer)?;
                buffer = buffer_;
                Ok((Section::Function(xs), buffer))
            }
            [4u8, ..] => {
                buffer = &buffer[1usize..];
                let (tabs, buffer_) = Sized::<Vec<Table>>::decode(buffer)?;
                buffer = buffer_;
                Ok((Section::Table(tabs), buffer))
            }
            [5u8, ..] => {
                buffer = &buffer[1usize..];
                let (mems, buffer_) = Sized::<Vec<Mem>>::decode(buffer)?;
                buffer = buffer_;
                Ok((Section::Mem(mems), buffer))
            }
            [6u8, ..] => {
                buffer = &buffer[1usize..];
                let (globs, buffer_) = Sized::<Vec<Global>>::decode(buffer)?;
                buffer = buffer_;
                Ok((Section::Global(globs), buffer))
            }
            [7u8, ..] => {
                buffer = &buffer[1usize..];
                let (exs, buffer_) = Sized::<Vec<Export>>::decode(buffer)?;
                buffer = buffer_;
                Ok((Section::Export(exs), buffer))
            }
            [8u8, ..] => {
                buffer = &buffer[1usize..];
                let (st, buffer_) = Sized::<FuncIdx>::decode(buffer)?;
                buffer = buffer_;
                Ok((Section::Start(st), buffer))
            }
            [9u8, ..] => {
                buffer = &buffer[1usize..];
                let (segs, buffer_) = Sized::<Vec<Elem>>::decode(buffer)?;
                buffer = buffer_;
                Ok((Section::Element(segs), buffer))
            }
            [10u8, ..] => {
                buffer = &buffer[1usize..];
                let (codes, buffer_) = Sized::<Vec<Code>>::decode(buffer)?;
                buffer = buffer_;
                Ok((Section::Code(codes), buffer))
            }
            [11u8, ..] => {
                buffer = &buffer[1usize..];
                let (segs, buffer_) = Sized::<Vec<Data>>::decode(buffer)?;
                buffer = buffer_;
                Ok((Section::Data(segs), buffer))
            }
            [12u8, ..] => {
                buffer = &buffer[1usize..];
                let (n, buffer_) = Sized::<u32>::decode(buffer)?;
                buffer = buffer_;
                Ok((Section::DataCount(n), buffer))
            }
            _ => Err(DecodeError::Error),
        }
    }
}
#[derive(:: core :: fmt :: Debug, :: core :: cmp :: PartialEq)]
pub struct Custom(Name, Repeated<u8>);
impl Encode for Custom {
    fn encode(&self, buffer: &mut Vec<u8>) {
        let Custom(name, bytes) = self;
        name.encode(buffer);
        bytes.encode(buffer);
    }
}
impl Decode for Custom {
    fn decode(mut buffer: &[u8]) -> DecodeResult<Custom> {
        let (name, buffer_) = Name::decode(buffer)?;
        buffer = buffer_;
        let (bytes, buffer_) = Repeated::<u8>::decode(buffer)?;
        buffer = buffer_;
        Ok((Custom(name, bytes), buffer))
    }
}
#[derive(:: core :: fmt :: Debug, :: core :: cmp :: PartialEq)]
pub struct FuncType(ResultType, ResultType);
impl Encode for FuncType {
    fn encode(&self, buffer: &mut Vec<u8>) {
        let FuncType(r1, r2) = self;
        buffer.push(96u8);
        r1.encode(buffer);
        r2.encode(buffer);
    }
}
impl Decode for FuncType {
    fn decode(mut buffer: &[u8]) -> DecodeResult<FuncType> {
        let (lit, buffer_) = u8::decode(buffer)?;
        if lit != 96u8 {
            return Err(DecodeError::Error);
        }
        buffer = buffer_;
        let (r1, buffer_) = ResultType::decode(buffer)?;
        buffer = buffer_;
        let (r2, buffer_) = ResultType::decode(buffer)?;
        buffer = buffer_;
        Ok((FuncType(r1, r2), buffer))
    }
}
#[derive(:: core :: fmt :: Debug, :: core :: cmp :: PartialEq)]
pub struct ResultType(Vec<ValType>);
impl Encode for ResultType {
    fn encode(&self, buffer: &mut Vec<u8>) {
        let ResultType(tys) = self;
        tys.encode(buffer);
    }
}
impl Decode for ResultType {
    fn decode(mut buffer: &[u8]) -> DecodeResult<ResultType> {
        let (tys, buffer_) = Vec::<ValType>::decode(buffer)?;
        buffer = buffer_;
        Ok((ResultType(tys), buffer))
    }
}
#[derive(:: core :: fmt :: Debug, :: core :: cmp :: PartialEq)]
pub enum ValType {
    I32(),
    I64(),
    F32(),
    F64(),
    V128(),
    FuncRef(),
    ExternRef(),
}
impl Encode for ValType {
    fn encode(&self, buffer: &mut Vec<u8>) {
        match self {
            ValType::I32() => {
                127u8.encode(buffer);
            }
            ValType::I64() => {
                126u8.encode(buffer);
            }
            ValType::F32() => {
                125u8.encode(buffer);
            }
            ValType::F64() => {
                124u8.encode(buffer);
            }
            ValType::V128() => {
                123u8.encode(buffer);
            }
            ValType::FuncRef() => {
                112u8.encode(buffer);
            }
            ValType::ExternRef() => {
                111u8.encode(buffer);
            }
        }
    }
}
impl Decode for ValType {
    fn decode(mut buffer: &[u8]) -> DecodeResult<ValType> {
        match buffer {
            [127u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((ValType::I32(), buffer))
            }
            [126u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((ValType::I64(), buffer))
            }
            [125u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((ValType::F32(), buffer))
            }
            [124u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((ValType::F64(), buffer))
            }
            [123u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((ValType::V128(), buffer))
            }
            [112u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((ValType::FuncRef(), buffer))
            }
            [111u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((ValType::ExternRef(), buffer))
            }
            _ => Err(DecodeError::Error),
        }
    }
}
#[derive(:: core :: fmt :: Debug, :: core :: cmp :: PartialEq)]
pub enum NumType {
    I32(),
    I64(),
    F32(),
    F64(),
}
impl Encode for NumType {
    fn encode(&self, buffer: &mut Vec<u8>) {
        match self {
            NumType::I32() => {
                127u8.encode(buffer);
            }
            NumType::I64() => {
                126u8.encode(buffer);
            }
            NumType::F32() => {
                125u8.encode(buffer);
            }
            NumType::F64() => {
                124u8.encode(buffer);
            }
        }
    }
}
impl Decode for NumType {
    fn decode(mut buffer: &[u8]) -> DecodeResult<NumType> {
        match buffer {
            [127u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((NumType::I32(), buffer))
            }
            [126u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((NumType::I64(), buffer))
            }
            [125u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((NumType::F32(), buffer))
            }
            [124u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((NumType::F64(), buffer))
            }
            _ => Err(DecodeError::Error),
        }
    }
}
#[derive(:: core :: fmt :: Debug, :: core :: cmp :: PartialEq)]
pub enum RefType {
    FuncRef(),
    ExternRef(),
}
impl Encode for RefType {
    fn encode(&self, buffer: &mut Vec<u8>) {
        match self {
            RefType::FuncRef() => {
                112u8.encode(buffer);
            }
            RefType::ExternRef() => {
                111u8.encode(buffer);
            }
        }
    }
}
impl Decode for RefType {
    fn decode(mut buffer: &[u8]) -> DecodeResult<RefType> {
        match buffer {
            [112u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((RefType::FuncRef(), buffer))
            }
            [111u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((RefType::ExternRef(), buffer))
            }
            _ => Err(DecodeError::Error),
        }
    }
}
#[derive(:: core :: fmt :: Debug, :: core :: cmp :: PartialEq)]
pub struct VecType();
impl Encode for VecType {
    fn encode(&self, buffer: &mut Vec<u8>) {
        let VecType() = self;
        buffer.push(123u8);
    }
}
impl Decode for VecType {
    fn decode(mut buffer: &[u8]) -> DecodeResult<VecType> {
        let (lit, buffer_) = u8::decode(buffer)?;
        if lit != 123u8 {
            return Err(DecodeError::Error);
        }
        buffer = buffer_;
        Ok((VecType(), buffer))
    }
}
#[derive(:: core :: fmt :: Debug, :: core :: cmp :: PartialEq)]
pub struct Import(Name, Name, ImportDesc);
impl Encode for Import {
    fn encode(&self, buffer: &mut Vec<u8>) {
        let Import(module, import_name, desc) = self;
        module.encode(buffer);
        import_name.encode(buffer);
        desc.encode(buffer);
    }
}
impl Decode for Import {
    fn decode(mut buffer: &[u8]) -> DecodeResult<Import> {
        let (module, buffer_) = Name::decode(buffer)?;
        buffer = buffer_;
        let (import_name, buffer_) = Name::decode(buffer)?;
        buffer = buffer_;
        let (desc, buffer_) = ImportDesc::decode(buffer)?;
        buffer = buffer_;
        Ok((Import(module, import_name, desc), buffer))
    }
}
#[derive(:: core :: fmt :: Debug, :: core :: cmp :: PartialEq)]
pub enum ImportDesc {
    Func(TypeIdx),
    Table(TableType),
    Mem(MemType),
    Global(GlobalType),
}
impl Encode for ImportDesc {
    fn encode(&self, buffer: &mut Vec<u8>) {
        match self {
            ImportDesc::Func(x) => {
                0u8.encode(buffer);
                x.encode(buffer);
            }
            ImportDesc::Table(tt) => {
                1u8.encode(buffer);
                tt.encode(buffer);
            }
            ImportDesc::Mem(mt) => {
                2u8.encode(buffer);
                mt.encode(buffer);
            }
            ImportDesc::Global(gt) => {
                3u8.encode(buffer);
                gt.encode(buffer);
            }
        }
    }
}
impl Decode for ImportDesc {
    fn decode(mut buffer: &[u8]) -> DecodeResult<ImportDesc> {
        match buffer {
            [0u8, ..] => {
                buffer = &buffer[1usize..];
                let (x, buffer_) = TypeIdx::decode(buffer)?;
                buffer = buffer_;
                Ok((ImportDesc::Func(x), buffer))
            }
            [1u8, ..] => {
                buffer = &buffer[1usize..];
                let (tt, buffer_) = TableType::decode(buffer)?;
                buffer = buffer_;
                Ok((ImportDesc::Table(tt), buffer))
            }
            [2u8, ..] => {
                buffer = &buffer[1usize..];
                let (mt, buffer_) = MemType::decode(buffer)?;
                buffer = buffer_;
                Ok((ImportDesc::Mem(mt), buffer))
            }
            [3u8, ..] => {
                buffer = &buffer[1usize..];
                let (gt, buffer_) = GlobalType::decode(buffer)?;
                buffer = buffer_;
                Ok((ImportDesc::Global(gt), buffer))
            }
            _ => Err(DecodeError::Error),
        }
    }
}
#[derive(:: core :: fmt :: Debug, :: core :: cmp :: PartialEq)]
pub struct TableType(RefType, Limits);
impl Encode for TableType {
    fn encode(&self, buffer: &mut Vec<u8>) {
        let TableType(et, lim) = self;
        et.encode(buffer);
        lim.encode(buffer);
    }
}
impl Decode for TableType {
    fn decode(mut buffer: &[u8]) -> DecodeResult<TableType> {
        let (et, buffer_) = RefType::decode(buffer)?;
        buffer = buffer_;
        let (lim, buffer_) = Limits::decode(buffer)?;
        buffer = buffer_;
        Ok((TableType(et, lim), buffer))
    }
}
#[derive(:: core :: fmt :: Debug, :: core :: cmp :: PartialEq)]
pub struct MemType(Limits);
impl Encode for MemType {
    fn encode(&self, buffer: &mut Vec<u8>) {
        let MemType(lim) = self;
        lim.encode(buffer);
    }
}
impl Decode for MemType {
    fn decode(mut buffer: &[u8]) -> DecodeResult<MemType> {
        let (lim, buffer_) = Limits::decode(buffer)?;
        buffer = buffer_;
        Ok((MemType(lim), buffer))
    }
}
#[derive(:: core :: fmt :: Debug, :: core :: cmp :: PartialEq)]
pub struct GlobalType(ValType, Mut);
impl Encode for GlobalType {
    fn encode(&self, buffer: &mut Vec<u8>) {
        let GlobalType(t, m) = self;
        t.encode(buffer);
        m.encode(buffer);
    }
}
impl Decode for GlobalType {
    fn decode(mut buffer: &[u8]) -> DecodeResult<GlobalType> {
        let (t, buffer_) = ValType::decode(buffer)?;
        buffer = buffer_;
        let (m, buffer_) = Mut::decode(buffer)?;
        buffer = buffer_;
        Ok((GlobalType(t, m), buffer))
    }
}
#[derive(:: core :: fmt :: Debug, :: core :: cmp :: PartialEq)]
pub enum Limits {
    Min(u32),
    MinMax(u32, u32),
}
impl Encode for Limits {
    fn encode(&self, buffer: &mut Vec<u8>) {
        match self {
            Limits::Min(n) => {
                0u8.encode(buffer);
                n.encode(buffer);
            }
            Limits::MinMax(n, m) => {
                1u8.encode(buffer);
                n.encode(buffer);
                m.encode(buffer);
            }
        }
    }
}
impl Decode for Limits {
    fn decode(mut buffer: &[u8]) -> DecodeResult<Limits> {
        match buffer {
            [0u8, ..] => {
                buffer = &buffer[1usize..];
                let (n, buffer_) = u32::decode(buffer)?;
                buffer = buffer_;
                Ok((Limits::Min(n), buffer))
            }
            [1u8, ..] => {
                buffer = &buffer[1usize..];
                let (n, buffer_) = u32::decode(buffer)?;
                buffer = buffer_;
                let (m, buffer_) = u32::decode(buffer)?;
                buffer = buffer_;
                Ok((Limits::MinMax(n, m), buffer))
            }
            _ => Err(DecodeError::Error),
        }
    }
}
#[derive(:: core :: fmt :: Debug, :: core :: cmp :: PartialEq)]
pub enum Mut {
    Const(),
    Mut(),
}
impl Encode for Mut {
    fn encode(&self, buffer: &mut Vec<u8>) {
        match self {
            Mut::Const() => {
                0u8.encode(buffer);
            }
            Mut::Mut() => {
                1u8.encode(buffer);
            }
        }
    }
}
impl Decode for Mut {
    fn decode(mut buffer: &[u8]) -> DecodeResult<Mut> {
        match buffer {
            [0u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Mut::Const(), buffer))
            }
            [1u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Mut::Mut(), buffer))
            }
            _ => Err(DecodeError::Error),
        }
    }
}
#[derive(:: core :: fmt :: Debug, :: core :: cmp :: PartialEq)]
pub struct Table(TableType);
impl Encode for Table {
    fn encode(&self, buffer: &mut Vec<u8>) {
        let Table(tt) = self;
        tt.encode(buffer);
    }
}
impl Decode for Table {
    fn decode(mut buffer: &[u8]) -> DecodeResult<Table> {
        let (tt, buffer_) = TableType::decode(buffer)?;
        buffer = buffer_;
        Ok((Table(tt), buffer))
    }
}
#[derive(:: core :: fmt :: Debug, :: core :: cmp :: PartialEq)]
pub struct Mem(MemType);
impl Encode for Mem {
    fn encode(&self, buffer: &mut Vec<u8>) {
        let Mem(mt) = self;
        mt.encode(buffer);
    }
}
impl Decode for Mem {
    fn decode(mut buffer: &[u8]) -> DecodeResult<Mem> {
        let (mt, buffer_) = MemType::decode(buffer)?;
        buffer = buffer_;
        Ok((Mem(mt), buffer))
    }
}
#[derive(:: core :: fmt :: Debug, :: core :: cmp :: PartialEq)]
pub struct Global(GlobalType, Expr);
impl Encode for Global {
    fn encode(&self, buffer: &mut Vec<u8>) {
        let Global(gt, e) = self;
        gt.encode(buffer);
        e.encode(buffer);
    }
}
impl Decode for Global {
    fn decode(mut buffer: &[u8]) -> DecodeResult<Global> {
        let (gt, buffer_) = GlobalType::decode(buffer)?;
        buffer = buffer_;
        let (e, buffer_) = Expr::decode(buffer)?;
        buffer = buffer_;
        Ok((Global(gt, e), buffer))
    }
}
#[derive(:: core :: fmt :: Debug, :: core :: cmp :: PartialEq)]
pub struct Export(Name, ExportDesc);
impl Encode for Export {
    fn encode(&self, buffer: &mut Vec<u8>) {
        let Export(nm, d) = self;
        nm.encode(buffer);
        d.encode(buffer);
    }
}
impl Decode for Export {
    fn decode(mut buffer: &[u8]) -> DecodeResult<Export> {
        let (nm, buffer_) = Name::decode(buffer)?;
        buffer = buffer_;
        let (d, buffer_) = ExportDesc::decode(buffer)?;
        buffer = buffer_;
        Ok((Export(nm, d), buffer))
    }
}
#[derive(:: core :: fmt :: Debug, :: core :: cmp :: PartialEq)]
pub enum ExportDesc {
    Func(FuncIdx),
    Table(TableIdx),
    Mem(MemIdx),
    Global(GlobalIdx),
}
impl Encode for ExportDesc {
    fn encode(&self, buffer: &mut Vec<u8>) {
        match self {
            ExportDesc::Func(x) => {
                0u8.encode(buffer);
                x.encode(buffer);
            }
            ExportDesc::Table(x) => {
                1u8.encode(buffer);
                x.encode(buffer);
            }
            ExportDesc::Mem(x) => {
                2u8.encode(buffer);
                x.encode(buffer);
            }
            ExportDesc::Global(x) => {
                3u8.encode(buffer);
                x.encode(buffer);
            }
        }
    }
}
impl Decode for ExportDesc {
    fn decode(mut buffer: &[u8]) -> DecodeResult<ExportDesc> {
        match buffer {
            [0u8, ..] => {
                buffer = &buffer[1usize..];
                let (x, buffer_) = FuncIdx::decode(buffer)?;
                buffer = buffer_;
                Ok((ExportDesc::Func(x), buffer))
            }
            [1u8, ..] => {
                buffer = &buffer[1usize..];
                let (x, buffer_) = TableIdx::decode(buffer)?;
                buffer = buffer_;
                Ok((ExportDesc::Table(x), buffer))
            }
            [2u8, ..] => {
                buffer = &buffer[1usize..];
                let (x, buffer_) = MemIdx::decode(buffer)?;
                buffer = buffer_;
                Ok((ExportDesc::Mem(x), buffer))
            }
            [3u8, ..] => {
                buffer = &buffer[1usize..];
                let (x, buffer_) = GlobalIdx::decode(buffer)?;
                buffer = buffer_;
                Ok((ExportDesc::Global(x), buffer))
            }
            _ => Err(DecodeError::Error),
        }
    }
}
#[derive(:: core :: fmt :: Debug, :: core :: cmp :: PartialEq)]
pub enum Elem {
    E0(Expr, Vec<FuncIdx>),
    E1(ElemKind, Vec<FuncIdx>),
    E2(TableIdx, Expr, ElemKind, Vec<FuncIdx>),
    E3(ElemKind, Vec<FuncIdx>),
    E4(Expr, Vec<Expr>),
    E5(RefType, Vec<Expr>),
    E6(TableIdx, Expr, RefType, Vec<Expr>),
    E7(RefType, Vec<Expr>),
}
impl Encode for Elem {
    fn encode(&self, buffer: &mut Vec<u8>) {
        match self {
            Elem::E0(e, y) => {
                0u8.encode(buffer);
                e.encode(buffer);
                y.encode(buffer);
            }
            Elem::E1(et, y) => {
                1u8.encode(buffer);
                et.encode(buffer);
                y.encode(buffer);
            }
            Elem::E2(x, e, et, y) => {
                2u8.encode(buffer);
                x.encode(buffer);
                e.encode(buffer);
                et.encode(buffer);
                y.encode(buffer);
            }
            Elem::E3(et, y) => {
                3u8.encode(buffer);
                et.encode(buffer);
                y.encode(buffer);
            }
            Elem::E4(e, els) => {
                4u8.encode(buffer);
                e.encode(buffer);
                els.encode(buffer);
            }
            Elem::E5(et, els) => {
                5u8.encode(buffer);
                et.encode(buffer);
                els.encode(buffer);
            }
            Elem::E6(x, e, et, els) => {
                6u8.encode(buffer);
                x.encode(buffer);
                e.encode(buffer);
                et.encode(buffer);
                els.encode(buffer);
            }
            Elem::E7(et, els) => {
                7u8.encode(buffer);
                et.encode(buffer);
                els.encode(buffer);
            }
        }
    }
}
impl Decode for Elem {
    fn decode(mut buffer: &[u8]) -> DecodeResult<Elem> {
        match buffer {
            [0u8, ..] => {
                buffer = &buffer[1usize..];
                let (e, buffer_) = Expr::decode(buffer)?;
                buffer = buffer_;
                let (y, buffer_) = Vec::<FuncIdx>::decode(buffer)?;
                buffer = buffer_;
                Ok((Elem::E0(e, y), buffer))
            }
            [1u8, ..] => {
                buffer = &buffer[1usize..];
                let (et, buffer_) = ElemKind::decode(buffer)?;
                buffer = buffer_;
                let (y, buffer_) = Vec::<FuncIdx>::decode(buffer)?;
                buffer = buffer_;
                Ok((Elem::E1(et, y), buffer))
            }
            [2u8, ..] => {
                buffer = &buffer[1usize..];
                let (x, buffer_) = TableIdx::decode(buffer)?;
                buffer = buffer_;
                let (e, buffer_) = Expr::decode(buffer)?;
                buffer = buffer_;
                let (et, buffer_) = ElemKind::decode(buffer)?;
                buffer = buffer_;
                let (y, buffer_) = Vec::<FuncIdx>::decode(buffer)?;
                buffer = buffer_;
                Ok((Elem::E2(x, e, et, y), buffer))
            }
            [3u8, ..] => {
                buffer = &buffer[1usize..];
                let (et, buffer_) = ElemKind::decode(buffer)?;
                buffer = buffer_;
                let (y, buffer_) = Vec::<FuncIdx>::decode(buffer)?;
                buffer = buffer_;
                Ok((Elem::E3(et, y), buffer))
            }
            [4u8, ..] => {
                buffer = &buffer[1usize..];
                let (e, buffer_) = Expr::decode(buffer)?;
                buffer = buffer_;
                let (els, buffer_) = Vec::<Expr>::decode(buffer)?;
                buffer = buffer_;
                Ok((Elem::E4(e, els), buffer))
            }
            [5u8, ..] => {
                buffer = &buffer[1usize..];
                let (et, buffer_) = RefType::decode(buffer)?;
                buffer = buffer_;
                let (els, buffer_) = Vec::<Expr>::decode(buffer)?;
                buffer = buffer_;
                Ok((Elem::E5(et, els), buffer))
            }
            [6u8, ..] => {
                buffer = &buffer[1usize..];
                let (x, buffer_) = TableIdx::decode(buffer)?;
                buffer = buffer_;
                let (e, buffer_) = Expr::decode(buffer)?;
                buffer = buffer_;
                let (et, buffer_) = RefType::decode(buffer)?;
                buffer = buffer_;
                let (els, buffer_) = Vec::<Expr>::decode(buffer)?;
                buffer = buffer_;
                Ok((Elem::E6(x, e, et, els), buffer))
            }
            [7u8, ..] => {
                buffer = &buffer[1usize..];
                let (et, buffer_) = RefType::decode(buffer)?;
                buffer = buffer_;
                let (els, buffer_) = Vec::<Expr>::decode(buffer)?;
                buffer = buffer_;
                Ok((Elem::E7(et, els), buffer))
            }
            _ => Err(DecodeError::Error),
        }
    }
}
#[derive(:: core :: fmt :: Debug, :: core :: cmp :: PartialEq)]
pub struct ElemKind();
impl Encode for ElemKind {
    fn encode(&self, buffer: &mut Vec<u8>) {
        let ElemKind() = self;
        buffer.push(0u8);
    }
}
impl Decode for ElemKind {
    fn decode(mut buffer: &[u8]) -> DecodeResult<ElemKind> {
        let (lit, buffer_) = u8::decode(buffer)?;
        if lit != 0u8 {
            return Err(DecodeError::Error);
        }
        buffer = buffer_;
        Ok((ElemKind(), buffer))
    }
}
#[derive(:: core :: fmt :: Debug, :: core :: cmp :: PartialEq)]
pub struct Code(u32, Func);
impl Encode for Code {
    fn encode(&self, buffer: &mut Vec<u8>) {
        let Code(size, code) = self;
        size.encode(buffer);
        code.encode(buffer);
    }
}
impl Decode for Code {
    fn decode(mut buffer: &[u8]) -> DecodeResult<Code> {
        let (size, buffer_) = u32::decode(buffer)?;
        buffer = buffer_;
        let (code, buffer_) = Func::decode(buffer)?;
        buffer = buffer_;
        Ok((Code(size, code), buffer))
    }
}
#[derive(:: core :: fmt :: Debug, :: core :: cmp :: PartialEq)]
pub struct Func(Vec<Locals>, Expr);
impl Encode for Func {
    fn encode(&self, buffer: &mut Vec<u8>) {
        let Func(locals, e) = self;
        locals.encode(buffer);
        e.encode(buffer);
    }
}
impl Decode for Func {
    fn decode(mut buffer: &[u8]) -> DecodeResult<Func> {
        let (locals, buffer_) = Vec::<Locals>::decode(buffer)?;
        buffer = buffer_;
        let (e, buffer_) = Expr::decode(buffer)?;
        buffer = buffer_;
        Ok((Func(locals, e), buffer))
    }
}
#[derive(:: core :: fmt :: Debug, :: core :: cmp :: PartialEq)]
pub struct Locals(u32, ValType);
impl Encode for Locals {
    fn encode(&self, buffer: &mut Vec<u8>) {
        let Locals(n, t) = self;
        n.encode(buffer);
        t.encode(buffer);
    }
}
impl Decode for Locals {
    fn decode(mut buffer: &[u8]) -> DecodeResult<Locals> {
        let (n, buffer_) = u32::decode(buffer)?;
        buffer = buffer_;
        let (t, buffer_) = ValType::decode(buffer)?;
        buffer = buffer_;
        Ok((Locals(n, t), buffer))
    }
}
#[derive(:: core :: fmt :: Debug, :: core :: cmp :: PartialEq)]
pub enum Data {
    D0(Expr, Vec<u8>),
    D1(Vec<u8>),
    D2(MemIdx, Expr, Vec<u8>),
}
impl Encode for Data {
    fn encode(&self, buffer: &mut Vec<u8>) {
        match self {
            Data::D0(e, bytes) => {
                0u8.encode(buffer);
                e.encode(buffer);
                bytes.encode(buffer);
            }
            Data::D1(bytes) => {
                1u8.encode(buffer);
                bytes.encode(buffer);
            }
            Data::D2(x, e, bytes) => {
                2u8.encode(buffer);
                x.encode(buffer);
                e.encode(buffer);
                bytes.encode(buffer);
            }
        }
    }
}
impl Decode for Data {
    fn decode(mut buffer: &[u8]) -> DecodeResult<Data> {
        match buffer {
            [0u8, ..] => {
                buffer = &buffer[1usize..];
                let (e, buffer_) = Expr::decode(buffer)?;
                buffer = buffer_;
                let (bytes, buffer_) = Vec::<u8>::decode(buffer)?;
                buffer = buffer_;
                Ok((Data::D0(e, bytes), buffer))
            }
            [1u8, ..] => {
                buffer = &buffer[1usize..];
                let (bytes, buffer_) = Vec::<u8>::decode(buffer)?;
                buffer = buffer_;
                Ok((Data::D1(bytes), buffer))
            }
            [2u8, ..] => {
                buffer = &buffer[1usize..];
                let (x, buffer_) = MemIdx::decode(buffer)?;
                buffer = buffer_;
                let (e, buffer_) = Expr::decode(buffer)?;
                buffer = buffer_;
                let (bytes, buffer_) = Vec::<u8>::decode(buffer)?;
                buffer = buffer_;
                Ok((Data::D2(x, e, bytes), buffer))
            }
            _ => Err(DecodeError::Error),
        }
    }
}
#[derive(:: core :: fmt :: Debug, :: core :: cmp :: PartialEq)]
pub struct TypeIdx(u32);
impl Encode for TypeIdx {
    fn encode(&self, buffer: &mut Vec<u8>) {
        let TypeIdx(x) = self;
        x.encode(buffer);
    }
}
impl Decode for TypeIdx {
    fn decode(mut buffer: &[u8]) -> DecodeResult<TypeIdx> {
        let (x, buffer_) = u32::decode(buffer)?;
        buffer = buffer_;
        Ok((TypeIdx(x), buffer))
    }
}
#[derive(:: core :: fmt :: Debug, :: core :: cmp :: PartialEq)]
pub struct TableIdx(u32);
impl Encode for TableIdx {
    fn encode(&self, buffer: &mut Vec<u8>) {
        let TableIdx(x) = self;
        x.encode(buffer);
    }
}
impl Decode for TableIdx {
    fn decode(mut buffer: &[u8]) -> DecodeResult<TableIdx> {
        let (x, buffer_) = u32::decode(buffer)?;
        buffer = buffer_;
        Ok((TableIdx(x), buffer))
    }
}
#[derive(:: core :: fmt :: Debug, :: core :: cmp :: PartialEq)]
pub struct FuncIdx(u32);
impl Encode for FuncIdx {
    fn encode(&self, buffer: &mut Vec<u8>) {
        let FuncIdx(x) = self;
        x.encode(buffer);
    }
}
impl Decode for FuncIdx {
    fn decode(mut buffer: &[u8]) -> DecodeResult<FuncIdx> {
        let (x, buffer_) = u32::decode(buffer)?;
        buffer = buffer_;
        Ok((FuncIdx(x), buffer))
    }
}
#[derive(:: core :: fmt :: Debug, :: core :: cmp :: PartialEq)]
pub struct MemIdx(u32);
impl Encode for MemIdx {
    fn encode(&self, buffer: &mut Vec<u8>) {
        let MemIdx(x) = self;
        x.encode(buffer);
    }
}
impl Decode for MemIdx {
    fn decode(mut buffer: &[u8]) -> DecodeResult<MemIdx> {
        let (x, buffer_) = u32::decode(buffer)?;
        buffer = buffer_;
        Ok((MemIdx(x), buffer))
    }
}
#[derive(:: core :: fmt :: Debug, :: core :: cmp :: PartialEq)]
pub struct GlobalIdx(u32);
impl Encode for GlobalIdx {
    fn encode(&self, buffer: &mut Vec<u8>) {
        let GlobalIdx(x) = self;
        x.encode(buffer);
    }
}
impl Decode for GlobalIdx {
    fn decode(mut buffer: &[u8]) -> DecodeResult<GlobalIdx> {
        let (x, buffer_) = u32::decode(buffer)?;
        buffer = buffer_;
        Ok((GlobalIdx(x), buffer))
    }
}
#[derive(:: core :: fmt :: Debug, :: core :: cmp :: PartialEq)]
pub struct LabelIdx(u32);
impl Encode for LabelIdx {
    fn encode(&self, buffer: &mut Vec<u8>) {
        let LabelIdx(x) = self;
        x.encode(buffer);
    }
}
impl Decode for LabelIdx {
    fn decode(mut buffer: &[u8]) -> DecodeResult<LabelIdx> {
        let (x, buffer_) = u32::decode(buffer)?;
        buffer = buffer_;
        Ok((LabelIdx(x), buffer))
    }
}
#[derive(:: core :: fmt :: Debug, :: core :: cmp :: PartialEq)]
pub struct LocalIdx(u32);
impl Encode for LocalIdx {
    fn encode(&self, buffer: &mut Vec<u8>) {
        let LocalIdx(x) = self;
        x.encode(buffer);
    }
}
impl Decode for LocalIdx {
    fn decode(mut buffer: &[u8]) -> DecodeResult<LocalIdx> {
        let (x, buffer_) = u32::decode(buffer)?;
        buffer = buffer_;
        Ok((LocalIdx(x), buffer))
    }
}
#[derive(:: core :: fmt :: Debug, :: core :: cmp :: PartialEq)]
pub struct ElemIdx(u32);
impl Encode for ElemIdx {
    fn encode(&self, buffer: &mut Vec<u8>) {
        let ElemIdx(x) = self;
        x.encode(buffer);
    }
}
impl Decode for ElemIdx {
    fn decode(mut buffer: &[u8]) -> DecodeResult<ElemIdx> {
        let (x, buffer_) = u32::decode(buffer)?;
        buffer = buffer_;
        Ok((ElemIdx(x), buffer))
    }
}
#[derive(:: core :: fmt :: Debug, :: core :: cmp :: PartialEq)]
pub struct DataIdx(u32);
impl Encode for DataIdx {
    fn encode(&self, buffer: &mut Vec<u8>) {
        let DataIdx(x) = self;
        x.encode(buffer);
    }
}
impl Decode for DataIdx {
    fn decode(mut buffer: &[u8]) -> DecodeResult<DataIdx> {
        let (x, buffer_) = u32::decode(buffer)?;
        buffer = buffer_;
        Ok((DataIdx(x), buffer))
    }
}
#[derive(:: core :: fmt :: Debug, :: core :: cmp :: PartialEq)]
pub struct Expr(Repeated<Instr>);
impl Encode for Expr {
    fn encode(&self, buffer: &mut Vec<u8>) {
        let Expr(instrs) = self;
        instrs.encode(buffer);
        buffer.push(11u8);
    }
}
impl Decode for Expr {
    fn decode(mut buffer: &[u8]) -> DecodeResult<Expr> {
        let (instrs, buffer_) = Repeated::<Instr>::decode(buffer)?;
        buffer = buffer_;
        let (lit, buffer_) = u8::decode(buffer)?;
        if lit != 11u8 {
            return Err(DecodeError::Error);
        }
        buffer = buffer_;
        Ok((Expr(instrs), buffer))
    }
}
#[derive(:: core :: fmt :: Debug, :: core :: cmp :: PartialEq)]
pub enum Instr {
    Unreachable(),
    Nop(),
    Block(i32, Repeated<Instr>),
    Loop(i32, Repeated<Instr>),
    If(i32, Repeated<Instr>, Else),
    Br(LabelIdx),
    BrIf(LabelIdx),
    BrTable(Vec<LabelIdx>, LabelIdx),
    Return(),
    Call(FuncIdx),
    CallIndirect(TypeIdx, TableIdx),
    RefNull(RefType),
    RefIsNull(),
    RefFunc(FuncIdx),
    Drop(),
    Select(),
    SelectTys(Vec<ValType>),
    LocalGet(LocalIdx),
    LocalSet(LocalIdx),
    LocalTee(LocalIdx),
    GlobalGet(LocalIdx),
    GlobalSet(LocalIdx),
    TableGet(TableIdx),
    TableSet(TableIdx),
    TableInit(ElemIdx, TableIdx),
    ElemDrop(ElemIdx),
    TableCopy(TableIdx, TableIdx),
    TableGrow(TableIdx),
    TableSize(TableIdx),
    TableFill(TableIdx),
    I32Load(MemArg),
    I64Load(MemArg),
    F32Load(MemArg),
    F64Load(MemArg),
    I32Load8S(MemArg),
    I32Load8U(MemArg),
    I32Load16S(MemArg),
    I32Load16U(MemArg),
    I64Load8S(MemArg),
    I64Load8U(MemArg),
    I64Load16S(MemArg),
    I64Load16U(MemArg),
    I64Load32S(MemArg),
    I64Load32U(MemArg),
    I32Store(MemArg),
    I64Store(MemArg),
    F32Store(MemArg),
    F64Store(MemArg),
    I32Store8(MemArg),
    I32Store16(MemArg),
    I64Store8(MemArg),
    I64Store16(MemArg),
    I64Store32(MemArg),
    MemorySize(),
    MemoryGrow(),
    MemoryInit(DataIdx),
    DataDrop(DataIdx),
    MemoryCopy(),
    MemoryFill(),
    I32Const(i32),
    I64Const(i64),
    F32Const(f32),
    F64Const(f64),
    I32Eqz(),
    I32Eq(),
    I32Ne(),
    I32LtS(),
    I32LtU(),
    I32GtS(),
    I32GtU(),
    I32LeS(),
    I32LeU(),
    I32GeS(),
    I32GeU(),
    I64EqZ(),
    I64Eq(),
    I64Ne(),
    I64LtS(),
    I64LtU(),
    I64GtS(),
    I64GtU(),
    I64LeS(),
    I64LeU(),
    I64GeS(),
    I64GeU(),
    F32Eq(),
    F32Ne(),
    F32Lt(),
    F32Gt(),
    F32Le(),
    F32Ge(),
    F64Eq(),
    F64Ne(),
    F64Lt(),
    F64Gt(),
    F64Le(),
    F64Ge(),
    I32Clz(),
    I32Ctz(),
    I32Popcnt(),
    I32Add(),
    I32Sub(),
    I32Mul(),
    I32DivS(),
    I32DivU(),
    I32RemS(),
    I32RemU(),
    I32And(),
    I32Or(),
    I32Xor(),
    I32Shl(),
    I32ShrS(),
    I32ShrU(),
    I32Rotl(),
    I32Rotr(),
    I64Clz(),
    I64Ctz(),
    I64Popcnt(),
    I64Add(),
    I64Sub(),
    I64Mul(),
    I64DivS(),
    I64DivU(),
    I64RemS(),
    I64RemU(),
    I64And(),
    I64Or(),
    I64Xor(),
    I64Shl(),
    I64ShrS(),
    I64ShrU(),
    I64Rotl(),
    I64Rotr(),
    F32Abs(),
    F32Neg(),
    F32Ceil(),
    F32Floor(),
    F32Trunc(),
    F32Nearest(),
    F32Sqrt(),
    F32Add(),
    F32Sub(),
    F32Mul(),
    F32Div(),
    F32Min(),
    F32Max(),
    F32Copysign(),
    F64Abs(),
    F64Neg(),
    F64Ceil(),
    F64Floor(),
    F64Trunc(),
    F64Nearest(),
    F64Sqrt(),
    F64Add(),
    F64Sub(),
    F64Mul(),
    F64Div(),
    F64Min(),
    F64Max(),
    F64Copysign(),
    I32WrapI64(),
    I32TruncF32S(),
    I32TruncF32U(),
    I32TruncF64S(),
    I32TruncF64U(),
    I64ExtendI32S(),
    I64ExtendI32U(),
    I64TruncF32S(),
    I64TruncF32U(),
    I64TruncF64S(),
    I64TruncF64U(),
    F32ConvertI32S(),
    F32ConvertI32U(),
    F32ConvertI64S(),
    F32ConvertI64U(),
    F32DemoteF64(),
    F64ConvertI32S(),
    F64ConvertI32U(),
    F64ConvertI64S(),
    F64ConvertI64U(),
    F64PromoteF32(),
    I32ReinterpretF32(),
    I64ReinterpretF64(),
    F32ReinterpretI32(),
    F64ReinterpretI64(),
    I32Extend8S(),
    I32Extend16S(),
    I64Extend8S(),
    I64Extend16S(),
    I64Extend32S(),
    I32TruncSatF32S(),
    I32TruncSatF32U(),
    I32TruncSatF64S(),
    I32TruncSatF64U(),
    I64TruncSatF32S(),
    I64TruncSatF32U(),
    I64TruncSatF64S(),
    I64TruncSatF64U(),
}
impl Encode for Instr {
    fn encode(&self, buffer: &mut Vec<u8>) {
        match self {
            Instr::Unreachable() => {
                0u8.encode(buffer);
            }
            Instr::Nop() => {
                1u8.encode(buffer);
            }
            Instr::Block(bt, instrs) => {
                2u8.encode(buffer);
                bt.encode(buffer);
                instrs.encode(buffer);
                11u8.encode(buffer);
            }
            Instr::Loop(bt, instrs) => {
                3u8.encode(buffer);
                bt.encode(buffer);
                instrs.encode(buffer);
                11u8.encode(buffer);
            }
            Instr::If(bt, instrs, else_) => {
                4u8.encode(buffer);
                bt.encode(buffer);
                instrs.encode(buffer);
                else_.encode(buffer);
            }
            Instr::Br(l) => {
                12u8.encode(buffer);
                l.encode(buffer);
            }
            Instr::BrIf(l) => {
                13u8.encode(buffer);
                l.encode(buffer);
            }
            Instr::BrTable(ls, ln) => {
                14u8.encode(buffer);
                ls.encode(buffer);
                ln.encode(buffer);
            }
            Instr::Return() => {
                15u8.encode(buffer);
            }
            Instr::Call(x) => {
                16u8.encode(buffer);
                x.encode(buffer);
            }
            Instr::CallIndirect(y, x) => {
                17u8.encode(buffer);
                y.encode(buffer);
                x.encode(buffer);
            }
            Instr::RefNull(t) => {
                208u8.encode(buffer);
                t.encode(buffer);
            }
            Instr::RefIsNull() => {
                209u8.encode(buffer);
            }
            Instr::RefFunc(x) => {
                210u8.encode(buffer);
                x.encode(buffer);
            }
            Instr::Drop() => {
                26u8.encode(buffer);
            }
            Instr::Select() => {
                27u8.encode(buffer);
            }
            Instr::SelectTys(tys) => {
                28u8.encode(buffer);
                tys.encode(buffer);
            }
            Instr::LocalGet(x) => {
                32u8.encode(buffer);
                x.encode(buffer);
            }
            Instr::LocalSet(x) => {
                33u8.encode(buffer);
                x.encode(buffer);
            }
            Instr::LocalTee(x) => {
                34u8.encode(buffer);
                x.encode(buffer);
            }
            Instr::GlobalGet(x) => {
                35u8.encode(buffer);
                x.encode(buffer);
            }
            Instr::GlobalSet(x) => {
                36u8.encode(buffer);
                x.encode(buffer);
            }
            Instr::TableGet(x) => {
                37u8.encode(buffer);
                x.encode(buffer);
            }
            Instr::TableSet(x) => {
                38u8.encode(buffer);
                x.encode(buffer);
            }
            Instr::TableInit(y, x) => {
                252u8.encode(buffer);
                12u32.encode(buffer);
                y.encode(buffer);
                x.encode(buffer);
            }
            Instr::ElemDrop(x) => {
                252u8.encode(buffer);
                13u32.encode(buffer);
                x.encode(buffer);
            }
            Instr::TableCopy(x, y) => {
                252u8.encode(buffer);
                14u32.encode(buffer);
                x.encode(buffer);
                y.encode(buffer);
            }
            Instr::TableGrow(x) => {
                252u8.encode(buffer);
                15u32.encode(buffer);
                x.encode(buffer);
            }
            Instr::TableSize(x) => {
                252u8.encode(buffer);
                16u32.encode(buffer);
                x.encode(buffer);
            }
            Instr::TableFill(x) => {
                252u8.encode(buffer);
                17u32.encode(buffer);
                x.encode(buffer);
            }
            Instr::I32Load(m) => {
                40u8.encode(buffer);
                m.encode(buffer);
            }
            Instr::I64Load(m) => {
                41u8.encode(buffer);
                m.encode(buffer);
            }
            Instr::F32Load(m) => {
                42u8.encode(buffer);
                m.encode(buffer);
            }
            Instr::F64Load(m) => {
                43u8.encode(buffer);
                m.encode(buffer);
            }
            Instr::I32Load8S(m) => {
                44u8.encode(buffer);
                m.encode(buffer);
            }
            Instr::I32Load8U(m) => {
                45u8.encode(buffer);
                m.encode(buffer);
            }
            Instr::I32Load16S(m) => {
                46u8.encode(buffer);
                m.encode(buffer);
            }
            Instr::I32Load16U(m) => {
                47u8.encode(buffer);
                m.encode(buffer);
            }
            Instr::I64Load8S(m) => {
                48u8.encode(buffer);
                m.encode(buffer);
            }
            Instr::I64Load8U(m) => {
                49u8.encode(buffer);
                m.encode(buffer);
            }
            Instr::I64Load16S(m) => {
                50u8.encode(buffer);
                m.encode(buffer);
            }
            Instr::I64Load16U(m) => {
                51u8.encode(buffer);
                m.encode(buffer);
            }
            Instr::I64Load32S(m) => {
                52u8.encode(buffer);
                m.encode(buffer);
            }
            Instr::I64Load32U(m) => {
                53u8.encode(buffer);
                m.encode(buffer);
            }
            Instr::I32Store(m) => {
                54u8.encode(buffer);
                m.encode(buffer);
            }
            Instr::I64Store(m) => {
                55u8.encode(buffer);
                m.encode(buffer);
            }
            Instr::F32Store(m) => {
                56u8.encode(buffer);
                m.encode(buffer);
            }
            Instr::F64Store(m) => {
                57u8.encode(buffer);
                m.encode(buffer);
            }
            Instr::I32Store8(m) => {
                58u8.encode(buffer);
                m.encode(buffer);
            }
            Instr::I32Store16(m) => {
                59u8.encode(buffer);
                m.encode(buffer);
            }
            Instr::I64Store8(m) => {
                60u8.encode(buffer);
                m.encode(buffer);
            }
            Instr::I64Store16(m) => {
                61u8.encode(buffer);
                m.encode(buffer);
            }
            Instr::I64Store32(m) => {
                62u8.encode(buffer);
                m.encode(buffer);
            }
            Instr::MemorySize() => {
                63u8.encode(buffer);
                0u8.encode(buffer);
            }
            Instr::MemoryGrow() => {
                64u8.encode(buffer);
                0u8.encode(buffer);
            }
            Instr::MemoryInit(x) => {
                252u8.encode(buffer);
                8u32.encode(buffer);
                x.encode(buffer);
                0u8.encode(buffer);
            }
            Instr::DataDrop(x) => {
                252u8.encode(buffer);
                9u32.encode(buffer);
                x.encode(buffer);
            }
            Instr::MemoryCopy() => {
                252u8.encode(buffer);
                10u32.encode(buffer);
                0u8.encode(buffer);
                0u8.encode(buffer);
            }
            Instr::MemoryFill() => {
                252u8.encode(buffer);
                11u32.encode(buffer);
                0u8.encode(buffer);
            }
            Instr::I32Const(n) => {
                65u8.encode(buffer);
                n.encode(buffer);
            }
            Instr::I64Const(n) => {
                66u8.encode(buffer);
                n.encode(buffer);
            }
            Instr::F32Const(z) => {
                67u8.encode(buffer);
                z.encode(buffer);
            }
            Instr::F64Const(z) => {
                68u8.encode(buffer);
                z.encode(buffer);
            }
            Instr::I32Eqz() => {
                69u8.encode(buffer);
            }
            Instr::I32Eq() => {
                70u8.encode(buffer);
            }
            Instr::I32Ne() => {
                71u8.encode(buffer);
            }
            Instr::I32LtS() => {
                72u8.encode(buffer);
            }
            Instr::I32LtU() => {
                73u8.encode(buffer);
            }
            Instr::I32GtS() => {
                74u8.encode(buffer);
            }
            Instr::I32GtU() => {
                75u8.encode(buffer);
            }
            Instr::I32LeS() => {
                76u8.encode(buffer);
            }
            Instr::I32LeU() => {
                77u8.encode(buffer);
            }
            Instr::I32GeS() => {
                78u8.encode(buffer);
            }
            Instr::I32GeU() => {
                79u8.encode(buffer);
            }
            Instr::I64EqZ() => {
                80u8.encode(buffer);
            }
            Instr::I64Eq() => {
                81u8.encode(buffer);
            }
            Instr::I64Ne() => {
                82u8.encode(buffer);
            }
            Instr::I64LtS() => {
                83u8.encode(buffer);
            }
            Instr::I64LtU() => {
                84u8.encode(buffer);
            }
            Instr::I64GtS() => {
                85u8.encode(buffer);
            }
            Instr::I64GtU() => {
                86u8.encode(buffer);
            }
            Instr::I64LeS() => {
                87u8.encode(buffer);
            }
            Instr::I64LeU() => {
                88u8.encode(buffer);
            }
            Instr::I64GeS() => {
                89u8.encode(buffer);
            }
            Instr::I64GeU() => {
                90u8.encode(buffer);
            }
            Instr::F32Eq() => {
                91u8.encode(buffer);
            }
            Instr::F32Ne() => {
                92u8.encode(buffer);
            }
            Instr::F32Lt() => {
                93u8.encode(buffer);
            }
            Instr::F32Gt() => {
                94u8.encode(buffer);
            }
            Instr::F32Le() => {
                95u8.encode(buffer);
            }
            Instr::F32Ge() => {
                96u8.encode(buffer);
            }
            Instr::F64Eq() => {
                97u8.encode(buffer);
            }
            Instr::F64Ne() => {
                98u8.encode(buffer);
            }
            Instr::F64Lt() => {
                99u8.encode(buffer);
            }
            Instr::F64Gt() => {
                100u8.encode(buffer);
            }
            Instr::F64Le() => {
                101u8.encode(buffer);
            }
            Instr::F64Ge() => {
                102u8.encode(buffer);
            }
            Instr::I32Clz() => {
                103u8.encode(buffer);
            }
            Instr::I32Ctz() => {
                104u8.encode(buffer);
            }
            Instr::I32Popcnt() => {
                105u8.encode(buffer);
            }
            Instr::I32Add() => {
                106u8.encode(buffer);
            }
            Instr::I32Sub() => {
                107u8.encode(buffer);
            }
            Instr::I32Mul() => {
                108u8.encode(buffer);
            }
            Instr::I32DivS() => {
                109u8.encode(buffer);
            }
            Instr::I32DivU() => {
                110u8.encode(buffer);
            }
            Instr::I32RemS() => {
                111u8.encode(buffer);
            }
            Instr::I32RemU() => {
                112u8.encode(buffer);
            }
            Instr::I32And() => {
                113u8.encode(buffer);
            }
            Instr::I32Or() => {
                114u8.encode(buffer);
            }
            Instr::I32Xor() => {
                115u8.encode(buffer);
            }
            Instr::I32Shl() => {
                116u8.encode(buffer);
            }
            Instr::I32ShrS() => {
                117u8.encode(buffer);
            }
            Instr::I32ShrU() => {
                118u8.encode(buffer);
            }
            Instr::I32Rotl() => {
                119u8.encode(buffer);
            }
            Instr::I32Rotr() => {
                120u8.encode(buffer);
            }
            Instr::I64Clz() => {
                121u8.encode(buffer);
            }
            Instr::I64Ctz() => {
                122u8.encode(buffer);
            }
            Instr::I64Popcnt() => {
                123u8.encode(buffer);
            }
            Instr::I64Add() => {
                124u8.encode(buffer);
            }
            Instr::I64Sub() => {
                125u8.encode(buffer);
            }
            Instr::I64Mul() => {
                126u8.encode(buffer);
            }
            Instr::I64DivS() => {
                127u8.encode(buffer);
            }
            Instr::I64DivU() => {
                128u8.encode(buffer);
            }
            Instr::I64RemS() => {
                129u8.encode(buffer);
            }
            Instr::I64RemU() => {
                130u8.encode(buffer);
            }
            Instr::I64And() => {
                131u8.encode(buffer);
            }
            Instr::I64Or() => {
                132u8.encode(buffer);
            }
            Instr::I64Xor() => {
                133u8.encode(buffer);
            }
            Instr::I64Shl() => {
                134u8.encode(buffer);
            }
            Instr::I64ShrS() => {
                135u8.encode(buffer);
            }
            Instr::I64ShrU() => {
                136u8.encode(buffer);
            }
            Instr::I64Rotl() => {
                137u8.encode(buffer);
            }
            Instr::I64Rotr() => {
                138u8.encode(buffer);
            }
            Instr::F32Abs() => {
                139u8.encode(buffer);
            }
            Instr::F32Neg() => {
                140u8.encode(buffer);
            }
            Instr::F32Ceil() => {
                141u8.encode(buffer);
            }
            Instr::F32Floor() => {
                142u8.encode(buffer);
            }
            Instr::F32Trunc() => {
                143u8.encode(buffer);
            }
            Instr::F32Nearest() => {
                144u8.encode(buffer);
            }
            Instr::F32Sqrt() => {
                145u8.encode(buffer);
            }
            Instr::F32Add() => {
                146u8.encode(buffer);
            }
            Instr::F32Sub() => {
                147u8.encode(buffer);
            }
            Instr::F32Mul() => {
                148u8.encode(buffer);
            }
            Instr::F32Div() => {
                149u8.encode(buffer);
            }
            Instr::F32Min() => {
                150u8.encode(buffer);
            }
            Instr::F32Max() => {
                151u8.encode(buffer);
            }
            Instr::F32Copysign() => {
                152u8.encode(buffer);
            }
            Instr::F64Abs() => {
                153u8.encode(buffer);
            }
            Instr::F64Neg() => {
                154u8.encode(buffer);
            }
            Instr::F64Ceil() => {
                155u8.encode(buffer);
            }
            Instr::F64Floor() => {
                156u8.encode(buffer);
            }
            Instr::F64Trunc() => {
                157u8.encode(buffer);
            }
            Instr::F64Nearest() => {
                158u8.encode(buffer);
            }
            Instr::F64Sqrt() => {
                159u8.encode(buffer);
            }
            Instr::F64Add() => {
                160u8.encode(buffer);
            }
            Instr::F64Sub() => {
                161u8.encode(buffer);
            }
            Instr::F64Mul() => {
                162u8.encode(buffer);
            }
            Instr::F64Div() => {
                163u8.encode(buffer);
            }
            Instr::F64Min() => {
                164u8.encode(buffer);
            }
            Instr::F64Max() => {
                165u8.encode(buffer);
            }
            Instr::F64Copysign() => {
                166u8.encode(buffer);
            }
            Instr::I32WrapI64() => {
                167u8.encode(buffer);
            }
            Instr::I32TruncF32S() => {
                168u8.encode(buffer);
            }
            Instr::I32TruncF32U() => {
                169u8.encode(buffer);
            }
            Instr::I32TruncF64S() => {
                170u8.encode(buffer);
            }
            Instr::I32TruncF64U() => {
                171u8.encode(buffer);
            }
            Instr::I64ExtendI32S() => {
                172u8.encode(buffer);
            }
            Instr::I64ExtendI32U() => {
                173u8.encode(buffer);
            }
            Instr::I64TruncF32S() => {
                174u8.encode(buffer);
            }
            Instr::I64TruncF32U() => {
                175u8.encode(buffer);
            }
            Instr::I64TruncF64S() => {
                176u8.encode(buffer);
            }
            Instr::I64TruncF64U() => {
                177u8.encode(buffer);
            }
            Instr::F32ConvertI32S() => {
                178u8.encode(buffer);
            }
            Instr::F32ConvertI32U() => {
                179u8.encode(buffer);
            }
            Instr::F32ConvertI64S() => {
                180u8.encode(buffer);
            }
            Instr::F32ConvertI64U() => {
                181u8.encode(buffer);
            }
            Instr::F32DemoteF64() => {
                182u8.encode(buffer);
            }
            Instr::F64ConvertI32S() => {
                183u8.encode(buffer);
            }
            Instr::F64ConvertI32U() => {
                184u8.encode(buffer);
            }
            Instr::F64ConvertI64S() => {
                185u8.encode(buffer);
            }
            Instr::F64ConvertI64U() => {
                186u8.encode(buffer);
            }
            Instr::F64PromoteF32() => {
                187u8.encode(buffer);
            }
            Instr::I32ReinterpretF32() => {
                188u8.encode(buffer);
            }
            Instr::I64ReinterpretF64() => {
                189u8.encode(buffer);
            }
            Instr::F32ReinterpretI32() => {
                190u8.encode(buffer);
            }
            Instr::F64ReinterpretI64() => {
                191u8.encode(buffer);
            }
            Instr::I32Extend8S() => {
                192u8.encode(buffer);
            }
            Instr::I32Extend16S() => {
                193u8.encode(buffer);
            }
            Instr::I64Extend8S() => {
                194u8.encode(buffer);
            }
            Instr::I64Extend16S() => {
                195u8.encode(buffer);
            }
            Instr::I64Extend32S() => {
                196u8.encode(buffer);
            }
            Instr::I32TruncSatF32S() => {
                252u8.encode(buffer);
                0u32.encode(buffer);
            }
            Instr::I32TruncSatF32U() => {
                252u8.encode(buffer);
                1u32.encode(buffer);
            }
            Instr::I32TruncSatF64S() => {
                252u8.encode(buffer);
                2u32.encode(buffer);
            }
            Instr::I32TruncSatF64U() => {
                252u8.encode(buffer);
                3u32.encode(buffer);
            }
            Instr::I64TruncSatF32S() => {
                252u8.encode(buffer);
                4u32.encode(buffer);
            }
            Instr::I64TruncSatF32U() => {
                252u8.encode(buffer);
                5u32.encode(buffer);
            }
            Instr::I64TruncSatF64S() => {
                252u8.encode(buffer);
                6u32.encode(buffer);
            }
            Instr::I64TruncSatF64U() => {
                252u8.encode(buffer);
                7u32.encode(buffer);
            }
        }
    }
}
impl Decode for Instr {
    fn decode(mut buffer: &[u8]) -> DecodeResult<Instr> {
        match buffer {
            [0u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::Unreachable(), buffer))
            }
            [1u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::Nop(), buffer))
            }
            [2u8, ..] => {
                buffer = &buffer[1usize..];
                let (bt, buffer_) = i32::decode(buffer)?;
                buffer = buffer_;
                let (instrs, buffer_) = Repeated::<Instr>::decode(buffer)?;
                buffer = buffer_;
                let (lit, buffer_) = u8::decode(buffer)?;
                if lit != 11u8 {
                    return Err(DecodeError::Error);
                }
                buffer = buffer_;
                Ok((Instr::Block(bt, instrs), buffer))
            }
            [3u8, ..] => {
                buffer = &buffer[1usize..];
                let (bt, buffer_) = i32::decode(buffer)?;
                buffer = buffer_;
                let (instrs, buffer_) = Repeated::<Instr>::decode(buffer)?;
                buffer = buffer_;
                let (lit, buffer_) = u8::decode(buffer)?;
                if lit != 11u8 {
                    return Err(DecodeError::Error);
                }
                buffer = buffer_;
                Ok((Instr::Loop(bt, instrs), buffer))
            }
            [4u8, ..] => {
                buffer = &buffer[1usize..];
                let (bt, buffer_) = i32::decode(buffer)?;
                buffer = buffer_;
                let (instrs, buffer_) = Repeated::<Instr>::decode(buffer)?;
                buffer = buffer_;
                let (else_, buffer_) = Else::decode(buffer)?;
                buffer = buffer_;
                Ok((Instr::If(bt, instrs, else_), buffer))
            }
            [12u8, ..] => {
                buffer = &buffer[1usize..];
                let (l, buffer_) = LabelIdx::decode(buffer)?;
                buffer = buffer_;
                Ok((Instr::Br(l), buffer))
            }
            [13u8, ..] => {
                buffer = &buffer[1usize..];
                let (l, buffer_) = LabelIdx::decode(buffer)?;
                buffer = buffer_;
                Ok((Instr::BrIf(l), buffer))
            }
            [14u8, ..] => {
                buffer = &buffer[1usize..];
                let (ls, buffer_) = Vec::<LabelIdx>::decode(buffer)?;
                buffer = buffer_;
                let (ln, buffer_) = LabelIdx::decode(buffer)?;
                buffer = buffer_;
                Ok((Instr::BrTable(ls, ln), buffer))
            }
            [15u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::Return(), buffer))
            }
            [16u8, ..] => {
                buffer = &buffer[1usize..];
                let (x, buffer_) = FuncIdx::decode(buffer)?;
                buffer = buffer_;
                Ok((Instr::Call(x), buffer))
            }
            [17u8, ..] => {
                buffer = &buffer[1usize..];
                let (y, buffer_) = TypeIdx::decode(buffer)?;
                buffer = buffer_;
                let (x, buffer_) = TableIdx::decode(buffer)?;
                buffer = buffer_;
                Ok((Instr::CallIndirect(y, x), buffer))
            }
            [208u8, ..] => {
                buffer = &buffer[1usize..];
                let (t, buffer_) = RefType::decode(buffer)?;
                buffer = buffer_;
                Ok((Instr::RefNull(t), buffer))
            }
            [209u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::RefIsNull(), buffer))
            }
            [210u8, ..] => {
                buffer = &buffer[1usize..];
                let (x, buffer_) = FuncIdx::decode(buffer)?;
                buffer = buffer_;
                Ok((Instr::RefFunc(x), buffer))
            }
            [26u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::Drop(), buffer))
            }
            [27u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::Select(), buffer))
            }
            [28u8, ..] => {
                buffer = &buffer[1usize..];
                let (tys, buffer_) = Vec::<ValType>::decode(buffer)?;
                buffer = buffer_;
                Ok((Instr::SelectTys(tys), buffer))
            }
            [32u8, ..] => {
                buffer = &buffer[1usize..];
                let (x, buffer_) = LocalIdx::decode(buffer)?;
                buffer = buffer_;
                Ok((Instr::LocalGet(x), buffer))
            }
            [33u8, ..] => {
                buffer = &buffer[1usize..];
                let (x, buffer_) = LocalIdx::decode(buffer)?;
                buffer = buffer_;
                Ok((Instr::LocalSet(x), buffer))
            }
            [34u8, ..] => {
                buffer = &buffer[1usize..];
                let (x, buffer_) = LocalIdx::decode(buffer)?;
                buffer = buffer_;
                Ok((Instr::LocalTee(x), buffer))
            }
            [35u8, ..] => {
                buffer = &buffer[1usize..];
                let (x, buffer_) = LocalIdx::decode(buffer)?;
                buffer = buffer_;
                Ok((Instr::GlobalGet(x), buffer))
            }
            [36u8, ..] => {
                buffer = &buffer[1usize..];
                let (x, buffer_) = LocalIdx::decode(buffer)?;
                buffer = buffer_;
                Ok((Instr::GlobalSet(x), buffer))
            }
            [37u8, ..] => {
                buffer = &buffer[1usize..];
                let (x, buffer_) = TableIdx::decode(buffer)?;
                buffer = buffer_;
                Ok((Instr::TableGet(x), buffer))
            }
            [38u8, ..] => {
                buffer = &buffer[1usize..];
                let (x, buffer_) = TableIdx::decode(buffer)?;
                buffer = buffer_;
                Ok((Instr::TableSet(x), buffer))
            }
            [252u8, 12u8, ..] => {
                buffer = &buffer[2usize..];
                let (y, buffer_) = ElemIdx::decode(buffer)?;
                buffer = buffer_;
                let (x, buffer_) = TableIdx::decode(buffer)?;
                buffer = buffer_;
                Ok((Instr::TableInit(y, x), buffer))
            }
            [252u8, 13u8, ..] => {
                buffer = &buffer[2usize..];
                let (x, buffer_) = ElemIdx::decode(buffer)?;
                buffer = buffer_;
                Ok((Instr::ElemDrop(x), buffer))
            }
            [252u8, 14u8, ..] => {
                buffer = &buffer[2usize..];
                let (x, buffer_) = TableIdx::decode(buffer)?;
                buffer = buffer_;
                let (y, buffer_) = TableIdx::decode(buffer)?;
                buffer = buffer_;
                Ok((Instr::TableCopy(x, y), buffer))
            }
            [252u8, 15u8, ..] => {
                buffer = &buffer[2usize..];
                let (x, buffer_) = TableIdx::decode(buffer)?;
                buffer = buffer_;
                Ok((Instr::TableGrow(x), buffer))
            }
            [252u8, 16u8, ..] => {
                buffer = &buffer[2usize..];
                let (x, buffer_) = TableIdx::decode(buffer)?;
                buffer = buffer_;
                Ok((Instr::TableSize(x), buffer))
            }
            [252u8, 17u8, ..] => {
                buffer = &buffer[2usize..];
                let (x, buffer_) = TableIdx::decode(buffer)?;
                buffer = buffer_;
                Ok((Instr::TableFill(x), buffer))
            }
            [40u8, ..] => {
                buffer = &buffer[1usize..];
                let (m, buffer_) = MemArg::decode(buffer)?;
                buffer = buffer_;
                Ok((Instr::I32Load(m), buffer))
            }
            [41u8, ..] => {
                buffer = &buffer[1usize..];
                let (m, buffer_) = MemArg::decode(buffer)?;
                buffer = buffer_;
                Ok((Instr::I64Load(m), buffer))
            }
            [42u8, ..] => {
                buffer = &buffer[1usize..];
                let (m, buffer_) = MemArg::decode(buffer)?;
                buffer = buffer_;
                Ok((Instr::F32Load(m), buffer))
            }
            [43u8, ..] => {
                buffer = &buffer[1usize..];
                let (m, buffer_) = MemArg::decode(buffer)?;
                buffer = buffer_;
                Ok((Instr::F64Load(m), buffer))
            }
            [44u8, ..] => {
                buffer = &buffer[1usize..];
                let (m, buffer_) = MemArg::decode(buffer)?;
                buffer = buffer_;
                Ok((Instr::I32Load8S(m), buffer))
            }
            [45u8, ..] => {
                buffer = &buffer[1usize..];
                let (m, buffer_) = MemArg::decode(buffer)?;
                buffer = buffer_;
                Ok((Instr::I32Load8U(m), buffer))
            }
            [46u8, ..] => {
                buffer = &buffer[1usize..];
                let (m, buffer_) = MemArg::decode(buffer)?;
                buffer = buffer_;
                Ok((Instr::I32Load16S(m), buffer))
            }
            [47u8, ..] => {
                buffer = &buffer[1usize..];
                let (m, buffer_) = MemArg::decode(buffer)?;
                buffer = buffer_;
                Ok((Instr::I32Load16U(m), buffer))
            }
            [48u8, ..] => {
                buffer = &buffer[1usize..];
                let (m, buffer_) = MemArg::decode(buffer)?;
                buffer = buffer_;
                Ok((Instr::I64Load8S(m), buffer))
            }
            [49u8, ..] => {
                buffer = &buffer[1usize..];
                let (m, buffer_) = MemArg::decode(buffer)?;
                buffer = buffer_;
                Ok((Instr::I64Load8U(m), buffer))
            }
            [50u8, ..] => {
                buffer = &buffer[1usize..];
                let (m, buffer_) = MemArg::decode(buffer)?;
                buffer = buffer_;
                Ok((Instr::I64Load16S(m), buffer))
            }
            [51u8, ..] => {
                buffer = &buffer[1usize..];
                let (m, buffer_) = MemArg::decode(buffer)?;
                buffer = buffer_;
                Ok((Instr::I64Load16U(m), buffer))
            }
            [52u8, ..] => {
                buffer = &buffer[1usize..];
                let (m, buffer_) = MemArg::decode(buffer)?;
                buffer = buffer_;
                Ok((Instr::I64Load32S(m), buffer))
            }
            [53u8, ..] => {
                buffer = &buffer[1usize..];
                let (m, buffer_) = MemArg::decode(buffer)?;
                buffer = buffer_;
                Ok((Instr::I64Load32U(m), buffer))
            }
            [54u8, ..] => {
                buffer = &buffer[1usize..];
                let (m, buffer_) = MemArg::decode(buffer)?;
                buffer = buffer_;
                Ok((Instr::I32Store(m), buffer))
            }
            [55u8, ..] => {
                buffer = &buffer[1usize..];
                let (m, buffer_) = MemArg::decode(buffer)?;
                buffer = buffer_;
                Ok((Instr::I64Store(m), buffer))
            }
            [56u8, ..] => {
                buffer = &buffer[1usize..];
                let (m, buffer_) = MemArg::decode(buffer)?;
                buffer = buffer_;
                Ok((Instr::F32Store(m), buffer))
            }
            [57u8, ..] => {
                buffer = &buffer[1usize..];
                let (m, buffer_) = MemArg::decode(buffer)?;
                buffer = buffer_;
                Ok((Instr::F64Store(m), buffer))
            }
            [58u8, ..] => {
                buffer = &buffer[1usize..];
                let (m, buffer_) = MemArg::decode(buffer)?;
                buffer = buffer_;
                Ok((Instr::I32Store8(m), buffer))
            }
            [59u8, ..] => {
                buffer = &buffer[1usize..];
                let (m, buffer_) = MemArg::decode(buffer)?;
                buffer = buffer_;
                Ok((Instr::I32Store16(m), buffer))
            }
            [60u8, ..] => {
                buffer = &buffer[1usize..];
                let (m, buffer_) = MemArg::decode(buffer)?;
                buffer = buffer_;
                Ok((Instr::I64Store8(m), buffer))
            }
            [61u8, ..] => {
                buffer = &buffer[1usize..];
                let (m, buffer_) = MemArg::decode(buffer)?;
                buffer = buffer_;
                Ok((Instr::I64Store16(m), buffer))
            }
            [62u8, ..] => {
                buffer = &buffer[1usize..];
                let (m, buffer_) = MemArg::decode(buffer)?;
                buffer = buffer_;
                Ok((Instr::I64Store32(m), buffer))
            }
            [63u8, 0u8, ..] => {
                buffer = &buffer[2usize..];
                Ok((Instr::MemorySize(), buffer))
            }
            [64u8, 0u8, ..] => {
                buffer = &buffer[2usize..];
                Ok((Instr::MemoryGrow(), buffer))
            }
            [252u8, 8u8, ..] => {
                buffer = &buffer[2usize..];
                let (x, buffer_) = DataIdx::decode(buffer)?;
                buffer = buffer_;
                let (lit, buffer_) = u8::decode(buffer)?;
                if lit != 0u8 {
                    return Err(DecodeError::Error);
                }
                buffer = buffer_;
                Ok((Instr::MemoryInit(x), buffer))
            }
            [252u8, 9u8, ..] => {
                buffer = &buffer[2usize..];
                let (x, buffer_) = DataIdx::decode(buffer)?;
                buffer = buffer_;
                Ok((Instr::DataDrop(x), buffer))
            }
            [252u8, 10u8, 0u8, 0u8, ..] => {
                buffer = &buffer[4usize..];
                Ok((Instr::MemoryCopy(), buffer))
            }
            [252u8, 11u8, 0u8, ..] => {
                buffer = &buffer[3usize..];
                Ok((Instr::MemoryFill(), buffer))
            }
            [65u8, ..] => {
                buffer = &buffer[1usize..];
                let (n, buffer_) = i32::decode(buffer)?;
                buffer = buffer_;
                Ok((Instr::I32Const(n), buffer))
            }
            [66u8, ..] => {
                buffer = &buffer[1usize..];
                let (n, buffer_) = i64::decode(buffer)?;
                buffer = buffer_;
                Ok((Instr::I64Const(n), buffer))
            }
            [67u8, ..] => {
                buffer = &buffer[1usize..];
                let (z, buffer_) = f32::decode(buffer)?;
                buffer = buffer_;
                Ok((Instr::F32Const(z), buffer))
            }
            [68u8, ..] => {
                buffer = &buffer[1usize..];
                let (z, buffer_) = f64::decode(buffer)?;
                buffer = buffer_;
                Ok((Instr::F64Const(z), buffer))
            }
            [69u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::I32Eqz(), buffer))
            }
            [70u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::I32Eq(), buffer))
            }
            [71u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::I32Ne(), buffer))
            }
            [72u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::I32LtS(), buffer))
            }
            [73u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::I32LtU(), buffer))
            }
            [74u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::I32GtS(), buffer))
            }
            [75u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::I32GtU(), buffer))
            }
            [76u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::I32LeS(), buffer))
            }
            [77u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::I32LeU(), buffer))
            }
            [78u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::I32GeS(), buffer))
            }
            [79u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::I32GeU(), buffer))
            }
            [80u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::I64EqZ(), buffer))
            }
            [81u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::I64Eq(), buffer))
            }
            [82u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::I64Ne(), buffer))
            }
            [83u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::I64LtS(), buffer))
            }
            [84u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::I64LtU(), buffer))
            }
            [85u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::I64GtS(), buffer))
            }
            [86u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::I64GtU(), buffer))
            }
            [87u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::I64LeS(), buffer))
            }
            [88u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::I64LeU(), buffer))
            }
            [89u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::I64GeS(), buffer))
            }
            [90u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::I64GeU(), buffer))
            }
            [91u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::F32Eq(), buffer))
            }
            [92u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::F32Ne(), buffer))
            }
            [93u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::F32Lt(), buffer))
            }
            [94u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::F32Gt(), buffer))
            }
            [95u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::F32Le(), buffer))
            }
            [96u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::F32Ge(), buffer))
            }
            [97u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::F64Eq(), buffer))
            }
            [98u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::F64Ne(), buffer))
            }
            [99u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::F64Lt(), buffer))
            }
            [100u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::F64Gt(), buffer))
            }
            [101u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::F64Le(), buffer))
            }
            [102u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::F64Ge(), buffer))
            }
            [103u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::I32Clz(), buffer))
            }
            [104u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::I32Ctz(), buffer))
            }
            [105u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::I32Popcnt(), buffer))
            }
            [106u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::I32Add(), buffer))
            }
            [107u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::I32Sub(), buffer))
            }
            [108u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::I32Mul(), buffer))
            }
            [109u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::I32DivS(), buffer))
            }
            [110u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::I32DivU(), buffer))
            }
            [111u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::I32RemS(), buffer))
            }
            [112u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::I32RemU(), buffer))
            }
            [113u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::I32And(), buffer))
            }
            [114u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::I32Or(), buffer))
            }
            [115u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::I32Xor(), buffer))
            }
            [116u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::I32Shl(), buffer))
            }
            [117u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::I32ShrS(), buffer))
            }
            [118u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::I32ShrU(), buffer))
            }
            [119u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::I32Rotl(), buffer))
            }
            [120u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::I32Rotr(), buffer))
            }
            [121u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::I64Clz(), buffer))
            }
            [122u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::I64Ctz(), buffer))
            }
            [123u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::I64Popcnt(), buffer))
            }
            [124u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::I64Add(), buffer))
            }
            [125u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::I64Sub(), buffer))
            }
            [126u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::I64Mul(), buffer))
            }
            [127u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::I64DivS(), buffer))
            }
            [128u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::I64DivU(), buffer))
            }
            [129u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::I64RemS(), buffer))
            }
            [130u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::I64RemU(), buffer))
            }
            [131u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::I64And(), buffer))
            }
            [132u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::I64Or(), buffer))
            }
            [133u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::I64Xor(), buffer))
            }
            [134u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::I64Shl(), buffer))
            }
            [135u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::I64ShrS(), buffer))
            }
            [136u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::I64ShrU(), buffer))
            }
            [137u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::I64Rotl(), buffer))
            }
            [138u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::I64Rotr(), buffer))
            }
            [139u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::F32Abs(), buffer))
            }
            [140u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::F32Neg(), buffer))
            }
            [141u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::F32Ceil(), buffer))
            }
            [142u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::F32Floor(), buffer))
            }
            [143u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::F32Trunc(), buffer))
            }
            [144u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::F32Nearest(), buffer))
            }
            [145u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::F32Sqrt(), buffer))
            }
            [146u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::F32Add(), buffer))
            }
            [147u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::F32Sub(), buffer))
            }
            [148u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::F32Mul(), buffer))
            }
            [149u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::F32Div(), buffer))
            }
            [150u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::F32Min(), buffer))
            }
            [151u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::F32Max(), buffer))
            }
            [152u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::F32Copysign(), buffer))
            }
            [153u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::F64Abs(), buffer))
            }
            [154u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::F64Neg(), buffer))
            }
            [155u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::F64Ceil(), buffer))
            }
            [156u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::F64Floor(), buffer))
            }
            [157u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::F64Trunc(), buffer))
            }
            [158u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::F64Nearest(), buffer))
            }
            [159u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::F64Sqrt(), buffer))
            }
            [160u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::F64Add(), buffer))
            }
            [161u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::F64Sub(), buffer))
            }
            [162u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::F64Mul(), buffer))
            }
            [163u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::F64Div(), buffer))
            }
            [164u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::F64Min(), buffer))
            }
            [165u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::F64Max(), buffer))
            }
            [166u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::F64Copysign(), buffer))
            }
            [167u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::I32WrapI64(), buffer))
            }
            [168u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::I32TruncF32S(), buffer))
            }
            [169u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::I32TruncF32U(), buffer))
            }
            [170u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::I32TruncF64S(), buffer))
            }
            [171u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::I32TruncF64U(), buffer))
            }
            [172u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::I64ExtendI32S(), buffer))
            }
            [173u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::I64ExtendI32U(), buffer))
            }
            [174u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::I64TruncF32S(), buffer))
            }
            [175u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::I64TruncF32U(), buffer))
            }
            [176u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::I64TruncF64S(), buffer))
            }
            [177u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::I64TruncF64U(), buffer))
            }
            [178u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::F32ConvertI32S(), buffer))
            }
            [179u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::F32ConvertI32U(), buffer))
            }
            [180u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::F32ConvertI64S(), buffer))
            }
            [181u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::F32ConvertI64U(), buffer))
            }
            [182u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::F32DemoteF64(), buffer))
            }
            [183u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::F64ConvertI32S(), buffer))
            }
            [184u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::F64ConvertI32U(), buffer))
            }
            [185u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::F64ConvertI64S(), buffer))
            }
            [186u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::F64ConvertI64U(), buffer))
            }
            [187u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::F64PromoteF32(), buffer))
            }
            [188u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::I32ReinterpretF32(), buffer))
            }
            [189u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::I64ReinterpretF64(), buffer))
            }
            [190u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::F32ReinterpretI32(), buffer))
            }
            [191u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::F64ReinterpretI64(), buffer))
            }
            [192u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::I32Extend8S(), buffer))
            }
            [193u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::I32Extend16S(), buffer))
            }
            [194u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::I64Extend8S(), buffer))
            }
            [195u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::I64Extend16S(), buffer))
            }
            [196u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Instr::I64Extend32S(), buffer))
            }
            [252u8, 0u8, ..] => {
                buffer = &buffer[2usize..];
                Ok((Instr::I32TruncSatF32S(), buffer))
            }
            [252u8, 1u8, ..] => {
                buffer = &buffer[2usize..];
                Ok((Instr::I32TruncSatF32U(), buffer))
            }
            [252u8, 2u8, ..] => {
                buffer = &buffer[2usize..];
                Ok((Instr::I32TruncSatF64S(), buffer))
            }
            [252u8, 3u8, ..] => {
                buffer = &buffer[2usize..];
                Ok((Instr::I32TruncSatF64U(), buffer))
            }
            [252u8, 4u8, ..] => {
                buffer = &buffer[2usize..];
                Ok((Instr::I64TruncSatF32S(), buffer))
            }
            [252u8, 5u8, ..] => {
                buffer = &buffer[2usize..];
                Ok((Instr::I64TruncSatF32U(), buffer))
            }
            [252u8, 6u8, ..] => {
                buffer = &buffer[2usize..];
                Ok((Instr::I64TruncSatF64S(), buffer))
            }
            [252u8, 7u8, ..] => {
                buffer = &buffer[2usize..];
                Ok((Instr::I64TruncSatF64U(), buffer))
            }
            _ => Err(DecodeError::Error),
        }
    }
}
#[derive(:: core :: fmt :: Debug, :: core :: cmp :: PartialEq)]
pub enum Else {
    NoElse(),
    Else(Repeated<Instr>),
}
impl Encode for Else {
    fn encode(&self, buffer: &mut Vec<u8>) {
        match self {
            Else::NoElse() => {
                11u8.encode(buffer);
            }
            Else::Else(instrs) => {
                5u8.encode(buffer);
                instrs.encode(buffer);
                11u8.encode(buffer);
            }
        }
    }
}
impl Decode for Else {
    fn decode(mut buffer: &[u8]) -> DecodeResult<Else> {
        match buffer {
            [11u8, ..] => {
                buffer = &buffer[1usize..];
                Ok((Else::NoElse(), buffer))
            }
            [5u8, ..] => {
                buffer = &buffer[1usize..];
                let (instrs, buffer_) = Repeated::<Instr>::decode(buffer)?;
                buffer = buffer_;
                let (lit, buffer_) = u8::decode(buffer)?;
                if lit != 11u8 {
                    return Err(DecodeError::Error);
                }
                buffer = buffer_;
                Ok((Else::Else(instrs), buffer))
            }
            _ => Err(DecodeError::Error),
        }
    }
}
#[derive(:: core :: fmt :: Debug, :: core :: cmp :: PartialEq)]
pub struct MemArg(u32, u32);
impl Encode for MemArg {
    fn encode(&self, buffer: &mut Vec<u8>) {
        let MemArg(align, offset) = self;
        align.encode(buffer);
        offset.encode(buffer);
    }
}
impl Decode for MemArg {
    fn decode(mut buffer: &[u8]) -> DecodeResult<MemArg> {
        let (align, buffer_) = u32::decode(buffer)?;
        buffer = buffer_;
        let (offset, buffer_) = u32::decode(buffer)?;
        buffer = buffer_;
        Ok((MemArg(align, offset), buffer))
    }
}
