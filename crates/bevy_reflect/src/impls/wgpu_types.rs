use crate::{self as bevy_reflect, ReflectDeserialize, ReflectSerialize, impl_reflect_opaque};

impl_reflect_opaque!(::wgpu_types::TextureFormat(
    Debug,
    Hash,
    PartialEq,
    Deserialize,
    Serialize,
));
