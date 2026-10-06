use crate::utils::{enum_byte_wrapper, enum_impls, python_test};

enum_byte_wrapper!(uds, Scaling, ScalingByte);
enum_impls!(uds, ScalingType, u8);
python_test!(uds, ScalingType, Bcd, Ascii);

/// Scaling high nibble, representing the type of data without its size. The size is given by the low nibble.
#[repr(u8)]
#[derive(strum::FromRepr, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "bin-proto", derive(bin_proto::BitEncode))]
#[cfg_attr(feature = "bin-proto", derive(bin_proto::BitDecode))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(feature = "display", derive(displaydoc::Display))]
#[cfg_attr(feature = "display", ignore_extra_doc_attributes)]
#[cfg_attr(feature = "iter", derive(strum::EnumIter))]
#[cfg_attr(feature = "pyo3", pyo3::pyclass(eq, eq_int, from_py_object))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ScalingType {
    /// Unsigned numeric integer. Must be followed by 1..4 bytes, given as a low nibble of the byte.
    UnsignedNumeric = 0x00,
    /// Signed numeric integer. Must be followed by 1..4 bytes, given as a low nibble of the byte.
    SignedNumeric = 0x10,
    /// Bit mapping encoding to set statuses, without mask
    BitMappingWithoutMask = 0x20,
    /// Bit mapping encoding to set statuses, with mask
    BitMappingWithMask = 0x30,
    /// Binary coded decimal encoding
    Bcd = 0x40,
    /// State encoded variable (Enum)
    StateEncodedVariable = 0x50,
    /// ASCII Text. Must be followed by 1..15 bytes, given as a low nibble of the byte.
    Ascii = 0x60,
    /// ANSI 754 signed floating point
    SignedFloatingPoint = 0x70,
    /// Multiple values data packet
    Packet = 0x80,
    /// Conversion formula
    Formula = 0x90,
    /// Unit of presentation format
    UnitOrFormat = 0xA0,
    /// Input / Output state encoding
    StateAndConnectionType = 0xB0,
}

/// Scaling value with both the [`ScalingType`] and the size of the data.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(feature = "pyo3", pyo3::pyclass(eq, from_py_object))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Scaling {
    typ: ScalingType,
    size: u8,
}

impl From<Scaling> for u8 {
    fn from(value: Scaling) -> Self {
        value.typ as Self | value.size
    }
}

impl Scaling {
    pub fn new(typ: ScalingType, size: u8) -> Result<Self, &'static str> {
        if size > 0x0F {
            return Err("Invalid size, expecting between 0 and 15.");
        }
        (typ as u8 | size).try_into()
    }

    /// Try to create [Self] from the raw representation
    #[must_use]
    pub fn from_repr(discriminant: u8) -> Option<Self> {
        Scaling::try_from(discriminant).ok()
    }
}

impl TryFrom<u8> for Scaling {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        let typ: ScalingType = (value & 0xF0).try_into()?;
        let size = value & 0x0F;
        match typ {
            ScalingType::UnsignedNumeric | ScalingType::SignedNumeric => {
                if !(1..=4).contains(&size) {
                    return Err("Invalid number of data bytes for a numeric type, expecting between 1 and 4.");
                }
            }
            ScalingType::Ascii => {
                if !(1..=15).contains(&size) {
                    return Err(
                        "Invalid number of data bytes for ASCII type, expecting between 1 and 15.",
                    );
                }
            }
            _ => {
                if size != 0 {
                    return Err("No data bytes are expected for this type.");
                }
            }
        }
        Ok(Self { typ, size })
    }
}

#[cfg(feature = "bin-proto")]
impl<E, Ctx> bin_proto::BitEncode<E, Ctx> for Scaling
where
    E: bin_proto::Endianness,
{
    fn encode<W>(&self, write: &mut W, ctx: &mut Ctx, (): ()) -> bin_proto::Result<()>
    where
        W: bin_proto::BitWrite + ?Sized,
    {
        <u8 as bin_proto::BitEncode<E, _, _>>::encode(&(*self).into(), write, ctx, ())
    }
}

#[cfg(feature = "bin-proto")]
impl<E, Ctx> bin_proto::BitDecode<E, Ctx> for Scaling
where
    E: bin_proto::Endianness,
{
    fn decode<R>(read: &mut R, ctx: &mut Ctx, (): ()) -> bin_proto::Result<Self>
    where
        R: bin_proto::BitRead + ?Sized,
    {
        <u8 as bin_proto::BitDecode<E, _, _>>::decode(read, ctx, ())?
            .try_into()
            .map_err(bin_proto::Error::msg)
    }
}

#[cfg(test)]
mod tests {
    use ScalingType::*;

    use super::*;

    #[test]
    fn test_scaling_byte() {
        assert_eq!(u8::from(Scaling::new(SignedNumeric, 4).unwrap()), 0x14);
        assert_eq!(u8::from(Scaling::new(SignedNumeric, 1).unwrap()), 0x11);
        assert!(Scaling::new(SignedNumeric, 0).is_err());
        assert!(Scaling::new(Bcd, 1).is_err());
    }

    #[test]
    #[cfg(feature = "bin-proto")]
    fn test_scaling_byte_encode() {
        let mut buf = [0; 1];

        assert_eq!(
            1,
            bin_proto::BitEncodeExt::encode_bytes_buf::<bin_proto::BigEndian>(
                &Scaling::new(SignedNumeric, 4).unwrap(),
                &mut buf
            )
            .unwrap()
        );
        assert_eq!(0x14, buf[0]);

        assert_eq!(
            1,
            bin_proto::BitEncodeExt::encode_bytes_buf::<bin_proto::BigEndian>(
                &Scaling::new(SignedNumeric, 1).unwrap(),
                &mut buf
            )
            .unwrap()
        );
        assert_eq!(0x11, buf[0]);
    }

    #[test]
    #[cfg(feature = "bin-proto")]
    fn test_scaling_byte_decode() {
        assert_eq!(
            Scaling::new(SignedNumeric, 4).unwrap(),
            bin_proto::BitDecodeExt::decode_all_bytes::<bin_proto::BigEndian>(&[0x14]).unwrap()
        );
        assert_eq!(
            Scaling::new(SignedNumeric, 1).unwrap(),
            bin_proto::BitDecodeExt::decode_all_bytes::<bin_proto::BigEndian>(&[0x11]).unwrap()
        );
        assert!(
            <Scaling as bin_proto::BitDecodeExt>::decode_all_bytes::<bin_proto::BigEndian>(&[0x10])
                .is_err()
        );
        assert!(
            <Scaling as bin_proto::BitDecodeExt>::decode_all_bytes::<bin_proto::BigEndian>(&[0x41])
                .is_err()
        );
    }
}
