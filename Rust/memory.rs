//! FFI 视图构建、深拷贝与总量预算，指针只在同步调用期借用。
use crate::abi::*;
use std::{
    any::Any,
    mem::{align_of, size_of},
};
#[derive(Default)]
pub struct Arena {
    allocations: Vec<Box<dyn Any>>,
    pub bytes: usize,
}
impl Arena {
    pub fn keep<T: 'static>(&mut self, value: T) -> *const T {
        let value = Box::new(value);
        let ptr = &*value as *const T;
        self.allocations.push(value);
        ptr
    }
    pub fn charge(&mut self, count: usize) -> Result<(), String> {
        self.bytes = self.bytes.checked_add(count).ok_or("视图长度溢出")?;
        if self.bytes > MAX_BYTES {
            return Err("视图超过 16 MiB".into());
        }
        Ok(())
    }
}
#[derive(Default)]
pub struct Budget {
    bytes: usize,
    depth: usize,
}
impl Budget {
    pub fn charge(&mut self, n: usize) -> Result<(), String> {
        self.bytes = self.bytes.checked_add(n).ok_or("输入长度溢出")?;
        if self.bytes > MAX_BYTES {
            return Err("输入超过 16 MiB".into());
        }
        Ok(())
    }
    pub fn enter(&mut self) -> Result<(), String> {
        self.depth += 1;
        if self.depth > MAX_DEPTH {
            return Err("输入超过深度限制".into());
        }
        Ok(())
    }
    pub fn leave(&mut self) {
        self.depth -= 1;
    }
}
pub trait Value: Sized {
    type Ffi: Copy + 'static;
    fn view(&self, arena: &mut Arena) -> Result<Self::Ffi, String>;
    /// # Safety
    /// 所有视图指针必须指向调用期有效内存。
    unsafe fn read(value: Self::Ffi, budget: &mut Budget) -> Result<Self, String>;
    /// 在复制结构前先验证其可读前缀。
    /// # Safety
    /// 指针必须指向相应类型的可读内存。
    unsafe fn read_ptr(ptr: *const Self::Ffi, budget: &mut Budget) -> Result<Self, String> {
        unsafe { Self::read(*checked(ptr)?, budget) }
    }
}
pub unsafe fn checked<'a, T>(ptr: *const T) -> Result<&'a T, String> {
    if ptr.is_null() || !(ptr as usize).is_multiple_of(align_of::<T>()) {
        return Err("无效或未对齐指针".into());
    }
    Ok(unsafe { &*ptr })
}
pub fn header<T>(value: Header) -> Result<(), String> {
    if value.abi_version != ABI_VERSION || value.struct_size < size_of::<T>() as u32 {
        return Err("ABI 版本或结构大小不兼容".into());
    }
    Ok(())
}
macro_rules! scalar {
    ($($t:ty),*) => {$ (
        impl Value for $t {
            type Ffi = $t;
            fn view(&self, _: &mut Arena) -> Result<$t,String> { Ok(*self) }
            unsafe fn read(value: $t, _: &mut Budget) -> Result<Self,String> { Ok(value) }
        }
    )*};
}
scalar!(u8, u32, u64, i64, f32, f64);
impl Value for bool {
    type Ffi = u32;
    fn view(&self, _: &mut Arena) -> Result<u32, String> {
        Ok(u32::from(*self))
    }
    unsafe fn read(value: u32, _: &mut Budget) -> Result<Self, String> {
        match value {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err("布尔值必须为 0/1".into()),
        }
    }
}
impl Value for String {
    type Ffi = StringView;
    fn view(&self, arena: &mut Arena) -> Result<StringView, String> {
        arena.charge(self.len())?;
        Ok(StringView::borrow(self))
    }
    unsafe fn read(value: StringView, budget: &mut Budget) -> Result<Self, String> {
        let len = usize::try_from(value.len).map_err(|_| "字符串长度溢出")?;
        budget.charge(len)?;
        if len == 0 {
            return Ok(String::new());
        }
        unsafe {
            checked(value.data)?;
        }
        let bytes = unsafe { std::slice::from_raw_parts(value.data, len) };
        std::str::from_utf8(bytes)
            .map(str::to_owned)
            .map_err(|e| e.to_string())
    }
}
impl<T: Value> Value for Vec<T> {
    type Ffi = Slice<T::Ffi>;
    fn view(&self, arena: &mut Arena) -> Result<Self::Ffi, String> {
        arena.charge(
            self.len()
                .checked_mul(size_of::<T::Ffi>())
                .ok_or("数组大小溢出")?,
        )?;
        let values = self
            .iter()
            .map(|v| v.view(arena))
            .collect::<Result<Vec<_>, _>>()?;
        let result = Slice::borrow(&values);
        arena.keep(values);
        Ok(result)
    }
    unsafe fn read(value: Self::Ffi, budget: &mut Budget) -> Result<Self, String> {
        let len = usize::try_from(value.len).map_err(|_| "数组长度溢出")?;
        let stride = usize::try_from(value.stride).map_err(|_| "数组步长溢出")?;
        if len == 0 {
            return Ok(Vec::new());
        }
        if stride < size_of::<T::Ffi>() || !stride.is_multiple_of(align_of::<T::Ffi>()) {
            return Err("数组步长无效".into());
        }
        budget.charge(len.checked_mul(stride).ok_or("数组大小溢出")?)?;
        unsafe {
            checked(value.data)?;
        }
        let mut result = Vec::with_capacity(len);
        for index in 0..len {
            let ptr = unsafe { value.data.cast::<u8>().add(index * stride).cast::<T::Ffi>() };
            result.push(unsafe { T::read_ptr(ptr, budget)? });
        }
        Ok(result)
    }
}
macro_rules! scalar_option {
    ($t:ty,$view:ty) => {
        impl Value for Option<$t> {
            type Ffi = $view;
            fn view(&self, _: &mut Arena) -> Result<Self::Ffi, String> {
                Ok(Self::Ffi {
                    has_value: u32::from(self.is_some()),
                    value: self.unwrap_or_default(),
                })
            }
            unsafe fn read(v: Self::Ffi, _: &mut Budget) -> Result<Self, String> {
                match v.has_value {
                    0 => Ok(None),
                    1 => Ok(Some(v.value)),
                    _ => Err("可选值标记无效".into()),
                }
            }
        }
    };
}
scalar_option!(u32, OptionalU32);
scalar_option!(i64, OptionalI64);
scalar_option!(u64, OptionalU64);
macro_rules! pointer_option {
    ($($t:ty),*) => {$ (
        impl Value for Option<$t> {
            type Ffi = *const <$t as Value>::Ffi;
            fn view(&self, arena: &mut Arena) -> Result<Self::Ffi,String> {
                match self { Some(v) => { let view=v.view(arena)?; Ok(arena.keep(view)) },None=>Ok(std::ptr::null()) }
            }
            unsafe fn read(v:Self::Ffi,b:&mut Budget)->Result<Self,String>{if v.is_null(){Ok(None)}else{Ok(Some(unsafe{<$t as Value>::read_ptr(v,b)?}))}}
        }
    )*};
}
pointer_option!(
    crate::proto::TransformPayload,
    crate::proto::MessageInput,
    crate::proto::MessageTypes,
    crate::proto::RuntimePaths,
    crate::proto::FrontendPayload,
    crate::proto::WebPayload,
    crate::proto::EventPayload,
    crate::proto::LogPayload,
    crate::proto::Subscription,
    crate::proto::WindowPayload,
    String,
    crate::proto::TextStyle,
    crate::proto::Image,
    crate::proto::Document,
    crate::proto::Selection,
    crate::proto::Message,
    crate::proto::MessageType,
    crate::proto::PluginError,
    crate::proto::DocumentEdit,
    crate::proto::UiContribution
);
macro_rules! ffi_struct {
    ($name:ident { $($field:ident: $t:ty),* }) => {
        #[repr(C)]
        #[derive(Clone,Copy)]
        pub struct $name { pub header: Header, $(pub $field: <$t as Value>::Ffi),* }
        impl Value for proto::$name {
            type Ffi = $name;
            fn view(&self, arena: &mut Arena) -> Result<Self::Ffi,String> {
                Ok($name {header: Header::of::<$name>(),$($field: self.$field.view(arena)?),*})
            }
            unsafe fn read_ptr(ptr:*const Self::Ffi,b:&mut Budget)->Result<Self,String>{
                let prefix=unsafe{checked(ptr.cast::<Header>())?};header::<$name>(*prefix)?;
                unsafe{Self::read(*checked(ptr)?,b)}
            }
            unsafe fn read(v:Self::Ffi,b:&mut Budget)->Result<Self,String>{
                header::<$name>(v.header)?; b.enter()?;
                let result=Ok(Self {$($field: unsafe {<$t as Value>::read(v.$field,b)?}),*});
                b.leave(); result
            }
        }
    };
}
pub(crate) use ffi_struct;
