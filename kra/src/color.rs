use std::marker::PhantomData;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[non_exhaustive]
pub enum Colorspace {
    /// Default RGBA colorspace.
    RGBA(RGBA),
    CMYKA(CMYKA),
    Alpha(Alpha),
}

impl Default for Colorspace {
    fn default() -> Self {
        Self::RGBA(RGBA())
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct RGBA();
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct CMYKA();
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Alpha();

pub trait ChannelCount {
    const CHANNELS: usize;
}

impl ChannelCount for RGBA {
    const CHANNELS: usize = 4;
}

impl ChannelCount for CMYKA {
    const CHANNELS: usize = 5;
}

impl ChannelCount for Alpha {
    const CHANNELS: usize = 1;
}

pub enum ChannelUnit {
    U8(U8),
    U16(U16),
    F32(F32),
}

impl Default for ChannelUnit {
    fn default() -> Self {
        Self::U8(U8())
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct U8();
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct U16();
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct F32();

pub trait ChannelType {
    type TYPE;
}

impl ChannelType for U8 {
    type TYPE = u8;
}

impl ChannelType for U16 {
    type TYPE = u16;
}

impl ChannelType for F32 {
    type TYPE = f32;
}

pub struct Color<C, U>
where
    C: ChannelCount,
    U: ChannelType,
    [(); C::CHANNELS]:,
{
    data: [U::TYPE; C::CHANNELS],
}
