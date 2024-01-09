//! Type definitions and their implementations for pallet-move and its RPC.

extern crate alloc;

use alloc::{
    string::{String, ToString},
    boxed::Box,
    vec::Vec,
};
use move_vm_backend::abi as mv_be_abi;
use move_core_types::{
    account_address::AccountAddress as OAccountAddress,
    identifier::{Identifier as OIdentifier, IdentStr as OIdentStr},
    language_storage::ModuleId as OModuleId,
};
use frame_support::pallet_prelude::{
    Encode, Decode, RuntimeDebug, TypeInfo,
};
use serde::{Deserialize, Serialize};

/// Pendant to `move_vm_backend::ModuleAbi` to avoid sustrate within move-vm-backend.
#[derive(Clone, Decode, Deserialize, Encode, Eq, Ord, PartialEq, PartialOrd, RuntimeDebug, Serialize, TypeInfo)]
pub struct ModuleAbi {
    /// Module ID.
    pub id: ModuleId,
    /// Friends.
    pub friends: Vec<Friend>,
    /// Structs.
    pub structs: Vec<Struct>,
    /// Functions.
    pub funcs: Vec<Function>,
}

impl From<mv_be_abi::ModuleAbi> for ModuleAbi {
    fn from(o_abi: mv_be_abi::ModuleAbi) -> ModuleAbi {
        ModuleAbi {
            id: o_abi.id.into(),
            friends: o_abi.friends.into_iter().map(|x| x.into()).collect(),
            structs: o_abi.structs.into_iter().map(|x| x.into()).collect(),
            funcs: o_abi.funcs.into_iter().map(|x| x.into()).collect(),
        }
    }
}

/// Pendant to `move_vm_backend::Friend` to avoid substrate within move-vm-backend.
#[derive(Clone, Decode, Deserialize, Encode, Eq, Ord, PartialEq, PartialOrd, RuntimeDebug, Serialize, TypeInfo)]
pub struct Friend {
    /// Address of the module.
    pub address: AccountAddress,
    /// Name of the module.
    pub name: Identifier,
}

impl From<mv_be_abi::Friend> for Friend {
    fn from(o_fr: mv_be_abi::Friend) -> Friend {
        Friend {
            address: o_fr.address.into(),
            name: o_fr.name.into(),
        }
    }
}

/// Pendant to `move_vm_backend::Function` to avoid substrate within move-vm-backend.
#[derive(Clone, Decode, Deserialize, Encode, Eq, Ord, PartialEq, PartialOrd, RuntimeDebug, Serialize, TypeInfo)]
pub struct Function {
    /// Name.
    pub name: Identifier,
    /// Visibility.
    pub visibility: FunctionVisibility,
    /// Generic type abilities.
    pub type_parameters: Vec<TypeAbilities>,
    /// Function arguments.
    pub parameters: Vec<Type>,
    /// Return types.
    pub returns: Vec<Type>,
}

impl From<mv_be_abi::Function> for Function {
    fn from(o_fn: mv_be_abi::Function) -> Function {
        Function {
            name: o_fn.name.into(),
            visibility: o_fn.visibility.into(),
            type_parameters: o_fn.type_parameters.into_iter().map(|x| x.into()).collect(),
            parameters: o_fn.parameters.into_iter().map(|x| x.into()).collect(),
            returns: o_fn.returns.into_iter().map(|x| x.into()).collect(),
        }
    }
}

/// Pendant to `move_vm_backend::Struct` to avoid substrate within move-vm-backend.
#[derive(Clone, Decode, Deserialize, Encode, Eq, Ord, PartialEq, PartialOrd, RuntimeDebug, Serialize, TypeInfo)]
pub struct Struct {
    /// Name identifier.
    pub name: Identifier,
    /// Generic type abilities.
    pub type_parameters: Vec<TypeAbilities>,
    /// Struct abilities.
    pub abilities: TypeAbilities,
    /// Struct elements.
    pub fields: Vec<Field>,
}

impl From<mv_be_abi::Struct> for Struct {
    fn from(o_struct: mv_be_abi::Struct) -> Struct {
        Struct {
            name: o_struct.name.into(),
            type_parameters: o_struct.type_parameters.into_iter().map(|x| x.into()).collect(),
            abilities: o_struct.abilities.into(),
            fields: o_struct.fields.into_iter().map(|x| x.into()).collect(),
        }
    }
}

/// Pendant to `move_vm_backend::FunctionVisibility` to avoid substrate within move-vm-backend.
#[derive(Clone, Decode, Deserialize, Encode, Eq, Ord, PartialEq, PartialOrd, RuntimeDebug, Serialize, TypeInfo)]
pub enum FunctionVisibility {                                                                        
    /// The procedure can be invoked anywhere `public`                                                                                     
    Public,                                                                                          
    /// The procedure can be invoked internally as well as by modules in the friend list             
    /// `public(friend)`
    Friend,
}

impl From<mv_be_abi::FunctionVisibility> for FunctionVisibility {
    fn from(o_vis: mv_be_abi::FunctionVisibility) -> FunctionVisibility {
        match o_vis {
            mv_be_abi::FunctionVisibility::Public => FunctionVisibility::Public,
            mv_be_abi::FunctionVisibility::Friend => FunctionVisibility::Friend,
        }
    }
}

/// Pendant to `move_vm_backend::StructDef` to avoid substrate within move-vm-backend.
#[derive(Clone, Decode, Deserialize, Encode, Eq, Ord, PartialEq, PartialOrd, RuntimeDebug, Serialize, TypeInfo)]
pub struct StructDef {
    /// Module ID.
    pub id: ModuleId,
    /// Name.
    pub name: Identifier,
    /// Struct fields.
    pub fields: Vec<Type>,
}

impl From<mv_be_abi::StructDef> for StructDef {
    fn from(o_df: mv_be_abi::StructDef) -> StructDef {
        StructDef {
            id: o_df.id.into(),
            name: o_df.name.into(),
            fields: o_df.fields.into_iter().map(|x| x.into()).collect(),
        }
    }
}

/// Pendant to `move_vm_backend::TypeAbilities` to avoid substrate within move-vm-backend.
#[derive(Clone, Decode, Deserialize, Encode, Eq, Ord, PartialEq, PartialOrd, RuntimeDebug, Serialize, TypeInfo)]
pub struct TypeAbilities {
    /// Abilities.
    pub abilities: Vec<TypeAbility>,
}

impl From<mv_be_abi::TypeAbilities> for TypeAbilities {
    fn from(o_ta: mv_be_abi::TypeAbilities) -> TypeAbilities {
        TypeAbilities {
            abilities: o_ta.abilities.into_iter().map(|x| x.into()).collect(),
        }
    }
}

/// Pendant to `move_vm_backend::Field` to avoid substrate within move-vm-backend.
#[derive(Clone, Decode, Deserialize, Encode, Eq, Ord, PartialEq, PartialOrd, RuntimeDebug, Serialize, TypeInfo)]
pub struct Field {
    /// Name.
    pub name: Identifier,
    /// Type.
    pub tp: Type,
}

impl From<mv_be_abi::Field> for Field {
    fn from(o_f: mv_be_abi::Field) -> Field {
        Field {
            name: o_f.name.into(),
            tp: o_f.tp.into(),
        }
    }
}

/// Pendant to `move_vm_backend::TypeAbility` to avoid substrate within move-vm-backend.
#[derive(Clone, Decode, Deserialize, Encode, Eq, Ord, PartialEq, PartialOrd, RuntimeDebug, Serialize, TypeInfo)]
pub enum TypeAbility {
    Copy,
    Drop,
    Store,
    Key,
}

impl From<mv_be_abi::TypeAbility> for TypeAbility {
    fn from(o_ta: mv_be_abi::TypeAbility) -> TypeAbility {
        match o_ta {
            mv_be_abi::TypeAbility::Copy => TypeAbility::Copy,
            mv_be_abi::TypeAbility::Drop => TypeAbility::Drop,
            mv_be_abi::TypeAbility::Store => TypeAbility::Store,
            mv_be_abi::TypeAbility::Key => TypeAbility::Key,
        }
    }
}

/// Pendant to `move_vm_backend::Type` to avoid substrate within move-vm-backend.
#[derive(Clone, Decode, Deserialize, Encode, Eq, Ord, PartialEq, PartialOrd, RuntimeDebug, Serialize, TypeInfo)]
pub enum Type {
    Bool,
    U8,
    U16,
    U32,
    U64,
    U128,
    U256,
    Address,
    Signer,
    Vector(Box<Type>),
    Struct(StructDef),
    Reference(Box<Type>),
    MutableReference(Box<Type>),
    TypeParameter(u16),
}

impl From<mv_be_abi::Type> for Type {
    fn from(tp: mv_be_abi::Type) -> Type {
        match tp {
            mv_be_abi::Type::Bool => Type::Bool,
            mv_be_abi::Type::U8 => Type::U8,
            mv_be_abi::Type::U16 => Type::U16,
            mv_be_abi::Type::U32 => Type::U32,
            mv_be_abi::Type::U64 => Type::U64,
            mv_be_abi::Type::U128 => Type::U128,
            mv_be_abi::Type::U256 => Type::U256,
            mv_be_abi::Type::Address => Type::Address,
            mv_be_abi::Type::Signer => Type::Signer,
            mv_be_abi::Type::Vector(bt) => {
                Type::Vector(Box::new(Type::from(*bt)))
            }
            mv_be_abi::Type::Struct(sd) => Type::Struct(sd.into()),
            mv_be_abi::Type::Reference(bt) => {
                Type::Reference(Box::new(Type::from(*bt)))
            }
            mv_be_abi::Type::MutableReference(bt) => {
                Type::MutableReference(Box::new(Type::from(*bt)))
            }
            mv_be_abi::Type::TypeParameter(u) => Type::TypeParameter(u),
        }
    }
}

/// Pendant to `move_core_types::ModuleId` to avoid substrate within move-vm-backend.
#[derive(Clone, Decode, Deserialize, Encode, Eq, Ord, PartialEq, PartialOrd, RuntimeDebug, Serialize, TypeInfo)]
pub struct ModuleId {
    pub address: AccountAddress,
    pub name: Identifier,
}

impl From<OModuleId> for ModuleId {
    fn from(o_id: OModuleId) -> ModuleId {
        ModuleId {
            address: (*o_id.address()).into(),
            name: o_id.name().into(),
        }
    }
}

/// Pendant to `move_core_types::AccountAddress` to avoid substrate within move-vm-backend.
#[derive(Clone, Decode, Deserialize, Encode, Eq, Ord, PartialEq, PartialOrd, RuntimeDebug, Serialize, TypeInfo)]
pub struct AccountAddress(pub [u8; 32]);

impl From<OAccountAddress> for AccountAddress {
    fn from(o_addr: OAccountAddress) -> AccountAddress {
        AccountAddress(o_addr.into_bytes())
    }
}

/// Pendant to `move_core_types::Identifier` to avoid substrate within move-vm-backend.
#[derive(Clone, Decode, Deserialize, Encode, Eq, Ord, PartialEq, PartialOrd, RuntimeDebug, Serialize, TypeInfo)]
pub struct Identifier(pub String);

impl From<OIdentifier> for Identifier {
    fn from(ident: OIdentifier) -> Identifier {
        Identifier(ident.into_string())
    }
}

impl From<&OIdentStr> for Identifier {
    fn from(o_str: &OIdentStr) -> Identifier {
        Identifier(o_str.as_str().to_string())
    }
}
