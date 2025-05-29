use std::{marker::PhantomData, ops::Deref};

use bevy::prelude::Component;
use godot::{
    classes::{Object, Resource},
    meta::{FromGodot, GodotConvert},
    obj::{
        AsDyn, Bounds, DynGd, Gd, GodotClass, Inherits, InstanceId, RawGd,
        bounds::{DeclUser, DynMemory},
    },
    sys,
};

#[derive(Debug, Component, Clone, Copy)]
pub struct ErasedGd {
    instance_id: InstanceId,
}

impl ErasedGd {
    pub fn get<T: Inherits<Object>>(&mut self) -> Gd<T> {
        self.try_get()
            .unwrap_or_else(|| panic!("failed to get godot ref as {}", std::any::type_name::<T>()))
    }

    /// # SAFETY
    /// The caller must uphold the contract of the constructors to ensure exclusive access
    pub fn try_get<T: Inherits<Object>>(&mut self) -> Option<Gd<T>> {
        Gd::try_from_instance_id(self.instance_id).ok()
    }

    /// # SAFETY
    /// When using ErasedGodotRef as a Bevy Resource or Component, do not create duplicate references
    /// to the same instance because Godot is not completely thread-safe.
    ///
    /// TODO
    /// Could these type bounds be more flexible to accomodate other types that are not ref-counted
    /// but don't inherit Node
    pub fn new<T: Inherits<Object>>(reference: Gd<T>) -> Self {
        Self {
            instance_id: reference.instance_id(),
        }
    }

    pub fn from_id(id: InstanceId) -> Self {
        Self { instance_id: id }
    }

    pub fn to_typed<T: GodotClass + Inherits<Object>>(self) -> TypedErasedGd<T> {
        TypedErasedGd::from_id(self.instance_id)
    }
}

#[derive(Debug, bevy::prelude::Resource)]
pub struct ErasedGdResource {
    resource_id: InstanceId,
}

/// used to access raw (RawGd) object
struct Gd_<T: GodotClass> {
    raw: RawGd<T>,
}

fn maybe_inc_ref<T: GodotClass>(gd: &mut Gd<T>) {
    let gd_: &mut Gd_<T> = unsafe { std::mem::transmute(gd) };
    <Object as Bounds>::DynMemory::maybe_inc_ref(&mut gd_.raw);
}

fn maybe_inc_ref_opt<T: GodotClass>(gd: &mut Option<Gd<T>>) {
    if let Some(gd) = gd {
        let gd_: &mut Gd_<T> = unsafe { std::mem::transmute(gd) };
        <Object as Bounds>::DynMemory::maybe_inc_ref(&mut gd_.raw);
    }
}

fn maybe_dec_ref<T: GodotClass>(gd: &mut Gd<T>) -> bool {
    let gd_: &mut Gd_<T> = unsafe { std::mem::transmute(gd) };
    unsafe { <Object as Bounds>::DynMemory::maybe_dec_ref(&mut gd_.raw) }
}

impl ErasedGdResource {
    pub fn get(&mut self) -> Gd<Resource> {
        self.try_get().unwrap()
    }

    pub fn try_get(&mut self) -> Option<Gd<Resource>> {
        Gd::try_from_instance_id(self.resource_id).ok()
    }

    pub fn new(mut reference: Gd<Resource>) -> Self {
        maybe_inc_ref(&mut reference);

        Self {
            resource_id: reference.instance_id(),
        }
    }
}

impl Clone for ErasedGdResource {
    fn clone(&self) -> Self {
        maybe_inc_ref_opt::<Resource>(&mut Gd::try_from_instance_id(self.resource_id).ok());

        Self {
            resource_id: self.resource_id,
        }
    }
}

impl Drop for ErasedGdResource {
    fn drop(&mut self) {
        let mut gd = self.get();
        let is_last = maybe_dec_ref(&mut gd); // may drop
        if is_last {
            unsafe {
                sys::interface_fn!(object_destroy)(gd.obj_sys());
            }
        }
    }
}

#[derive(Debug, Component)]
pub struct TypedErasedGd<T: GodotClass + Inherits<Object>> {
    instance: ErasedGd,
    _data: PhantomData<fn() -> T>,
}

impl<T: Inherits<Object>> Clone for TypedErasedGd<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: Inherits<Object>> Copy for TypedErasedGd<T> {}

impl<T: GodotClass + Inherits<Object>> TypedErasedGd<T> {
    pub fn new(reference: Gd<T>) -> Self {
        Self {
            instance: ErasedGd::new(reference),
            _data: PhantomData,
        }
    }

    pub fn from_id(id: InstanceId) -> Self {
        Self {
            instance: ErasedGd::from_id(id),
            _data: PhantomData,
        }
    }

    pub fn get(&mut self) -> Gd<T> {
        self.instance.get::<T>()
    }

    pub fn try_get(&mut self) -> Option<Gd<T>> {
        self.instance.try_get::<T>()
    }

    pub fn erase_type(self) -> ErasedGd {
        self.instance
    }
}

impl<T: GodotClass + Inherits<Object>> Deref for TypedErasedGd<T> {
    type Target = ErasedGd;

    fn deref(&self) -> &Self::Target {
        &self.instance
    }
}

impl<T: GodotClass + Inherits<Object>> From<Gd<T>> for TypedErasedGd<T> {
    fn from(value: Gd<T>) -> Self {
        Self::new(value)
    }
}

impl<T: GodotClass + Inherits<Object>> FromGodot for TypedErasedGd<T> {
    fn try_from_godot(via: Self::Via) -> Result<Self, godot::prelude::ConvertError> {
        Ok(Self::new(via))
    }
}

impl<T: GodotClass + Inherits<Object>> GodotConvert for TypedErasedGd<T> {
    type Via = Gd<T>;
}

#[derive(Debug, Component)]
pub struct DynErasedGd<D: ?Sized> {
    instance: ErasedGd,
    _data: PhantomData<fn() -> D>,
}
impl<T: Inherits<Object>> Clone for DynErasedGd<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: Inherits<Object>> Copy for DynErasedGd<T> {}

impl<D: ?Sized> DynErasedGd<D> {
    pub fn new_dyn<T: GodotClass + Inherits<Object>>(reference: DynGd<T, D>) -> Self {
        Self {
            instance: ErasedGd::new((*reference).clone()),
            _data: PhantomData,
        }
    }

    pub fn new<T: GodotClass + Inherits<Object>>(reference: Gd<T>) -> Self {
        Self {
            instance: ErasedGd::new(reference),
            _data: PhantomData,
        }
    }

    pub fn from_id(id: InstanceId) -> Self {
        Self {
            instance: ErasedGd::from_id(id),
            _data: PhantomData,
        }
    }

    pub fn get<T: GodotClass + Inherits<Object> + AsDyn<D> + Bounds<Declarer = DeclUser>>(
        &mut self,
    ) -> DynGd<T, D> {
        self.instance.get::<T>().into_dyn::<D>()
    }

    pub fn try_get<T: GodotClass + Inherits<Object> + AsDyn<D> + Bounds<Declarer = DeclUser>>(
        &mut self,
    ) -> Option<DynGd<T, D>> {
        self.instance.try_get::<T>().map(|i| i.into_dyn::<D>())
    }

    pub fn erase_type(self) -> ErasedGd {
        self.instance
    }
}

impl<T: GodotClass + Inherits<Object>> Deref for DynErasedGd<T> {
    type Target = ErasedGd;

    fn deref(&self) -> &Self::Target {
        &self.instance
    }
}
