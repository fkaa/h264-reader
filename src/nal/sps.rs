
use crate::rbsp::RbspBitReader;
use super::NalHandler;
use super::NalHeader;
use bitreader;
use crate::Context;
use crate::rbsp::RbspBitReaderError;
use std::{marker, fmt};
use crate::nal::pps::ParamSetId;
use crate::nal::pps::ParamSetIdError;
use std::fmt::Debug;

#[derive(Debug, PartialEq)]
pub enum SpsError {
    /// Signals that bit_depth_luma_minus8 was greater than the max value, 6
    BitDepthOutOfRange(u32),
    ReaderError(bitreader::BitReaderError),
    RbspReaderError(RbspBitReaderError),
    PicOrderCnt(PicOrderCntError),
    /// log2_max_frame_num_minus4 must be between 0 and 12
    Log2MaxFrameNumMinus4OutOfRange(u32),
    BadSeqParamSetId(ParamSetIdError),
    /// A field in the bitstream had a value too large for a subsequent calculation
    FieldValueTooLarge { name: &'static str, value: u32 },
    /// The frame-cropping values are too large vs. the coded picture size,
    CroppingError(FrameCropping),
}

impl From<bitreader::BitReaderError> for SpsError {
    fn from(e: bitreader::BitReaderError) -> Self {
        SpsError::ReaderError(e)
    }
}

impl From<RbspBitReaderError> for SpsError {
    fn from(e: RbspBitReaderError) -> Self {
        SpsError::RbspReaderError(e)
    }
}

pub struct SeqParameterSetNalHandler<Ctx> {
    buf: Vec<u8>,
    phantom: marker::PhantomData<Ctx>
}

impl<Ctx> Default for SeqParameterSetNalHandler<Ctx> {
    fn default() -> Self {
        SeqParameterSetNalHandler {
            buf: Vec::new(),
            phantom: marker::PhantomData,
        }
    }
}
impl<Ctx> NalHandler for SeqParameterSetNalHandler<Ctx> {
    type Ctx = Ctx;

    fn start(&mut self, _ctx: &mut Context<Ctx>, header: NalHeader) {
        assert_eq!(header.nal_unit_type(), super::UnitType::SeqParameterSet);
    }

    fn push(&mut self, _ctx: &mut Context<Ctx>, buf: &[u8]) {
        self.buf.extend_from_slice(buf);
    }

    fn end(&mut self, ctx: &mut Context<Ctx>) {
        let sps = SeqParameterSet::from_bytes(&self.buf[..]);
        self.buf.clear();
        if let Ok(sps) = sps {
            ctx.put_seq_param_set(sps);
        }
    }
}

#[derive(Debug)]
pub enum Profile {
    Unknown(u8),
    Baseline,
    Main,
    High,
    High422,
    High10,
    High444,
    Extended,
    ScalableBase,
    ScalableHigh,
    MultiviewHigh,
    StereoHigh,
    MFCDepthHigh,
    MultiviewDepthHigh,
    EnhancedMultiviewDepthHigh,
}

impl Profile {
    pub fn from_profile_idc(profile_idc: ProfileIdc) -> Profile {
        // TODO: accept constraint_flags too, as Level does?
        match profile_idc.0 {
            66  => Profile::Baseline,
            77  => Profile::Main,
            100 => Profile::High,
            122 => Profile::High422,
            110 => Profile::High10,
            144 => Profile::High444,
            88  => Profile::Extended,
            83  => Profile::ScalableBase,
            86  => Profile::ScalableHigh,
            118 => Profile::MultiviewHigh,
            128 => Profile::StereoHigh,
            135 => Profile::MFCDepthHigh,
            138 => Profile::MultiviewDepthHigh,
            139 => Profile::EnhancedMultiviewDepthHigh,
            other   => Profile::Unknown(other),
        }
    }
    pub fn profile_idc(&self) -> u8 {
        match *self {
            Profile::Baseline                   => 66,
            Profile::Main                       => 77,
            Profile::High                       => 100,
            Profile::High422                    => 122,
            Profile::High10                     => 110,
            Profile::High444                    => 144,
            Profile::Extended                   => 88,
            Profile::ScalableBase               => 83,
            Profile::ScalableHigh               => 86,
            Profile::MultiviewHigh              => 118,
            Profile::StereoHigh                 => 128,
            Profile::MFCDepthHigh               => 135,
            Profile::MultiviewDepthHigh         => 138,
            Profile::EnhancedMultiviewDepthHigh => 139,
            Profile::Unknown(profile_idc)       => profile_idc,
        }
    }
}

#[derive(Copy, Clone)]
pub struct ConstraintFlags(u8);
impl From<u8> for ConstraintFlags {
    fn from(v: u8) -> Self {
        ConstraintFlags(v)
    }
}
impl ConstraintFlags {
    pub fn flag0(self) -> bool { self.0 & 0b1000_0000 != 0 }
    pub fn flag1(self) -> bool { self.0 & 0b0100_0000 != 0 }
    pub fn flag2(self) -> bool { self.0 & 0b0010_0000 != 0 }
    pub fn flag3(self) -> bool { self.0 & 0b0001_0000 != 0 }
    pub fn flag4(self) -> bool { self.0 & 0b0000_1000 != 0 }
    pub fn flag5(self) -> bool { self.0 & 0b0000_0100 != 0 }
    pub fn reserved_zero_two_bits(self) -> u8 { self.0 & 0b0000_0011 }
}
impl Debug for ConstraintFlags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        f.debug_struct("ConstraintFlags")
            .field("flag0", &self.flag0())
            .field("flag1", &self.flag1())
            .field("flag2", &self.flag2())
            .field("flag3", &self.flag3())
            .field("flag4", &self.flag4())
            .field("flag5", &self.flag5())
            .field("reserved_zero_two_bits", &self.reserved_zero_two_bits())
            .finish()
    }
}

#[derive(Debug, PartialEq)]
#[allow(non_camel_case_types)]
pub enum Level {
    Unknown(u8),
    L1,
    L1_b,
    L1_1,
    L1_2,
    L1_3,
    L2,
    L2_1,
    L2_2,
    L3,
    L3_1,
    L3_2,
    L4,
    L4_1,
    L4_2,
    L5,
    L5_1,
    L5_2,
}
impl Level {
    pub fn from_constraint_flags_and_level_idc(constraint_flags: ConstraintFlags, level_idc: u8) -> Level {
        match level_idc {
            10 => Level::L1,
            11 => {
                if constraint_flags.flag3() {
                    Level::L1_b
                } else {
                    Level::L1_1
                }
            },
            12 => Level::L1_2,
            13 => Level::L1_3,
            20 => Level::L2,
            21 => Level::L2_1,
            22 => Level::L2_2,
            30 => Level::L3,
            31 => Level::L3_1,
            32 => Level::L3_2,
            40 => Level::L4,
            41 => Level::L4_1,
            42 => Level::L4_2,
            50 => Level::L5,
            51 => Level::L5_1,
            52 => Level::L5_2,
            _  => Level::Unknown(level_idc)
        }
    }
    pub fn level_idc(&self) -> u8 {
        match *self {
            Level::L1    => 10,
            Level::L1_1 | Level::L1_b  => 11,
            Level::L1_2  => 12,
            Level::L1_3  => 13,
            Level::L2    => 20,
            Level::L2_1  => 21,
            Level::L2_2  => 22,
            Level::L3    => 30,
            Level::L3_1  => 31,
            Level::L3_2  => 32,
            Level::L4    => 40,
            Level::L4_1  => 41,
            Level::L4_2  => 42,
            Level::L5    => 50,
            Level::L5_1  => 51,
            Level::L5_2  => 52,
            Level::Unknown(level_idc) => level_idc,
        }
    }
}

#[derive(Debug,PartialEq,Clone,Copy)]
pub enum ChromaFormat {
    Monochrome,
    YUV420,
    YUV422,
    YUV444,
    Invalid(u32),
}
impl ChromaFormat {
    fn from_chroma_format_idc(chroma_format_idc: u32) -> ChromaFormat{
        match chroma_format_idc {
            0 => ChromaFormat::Monochrome,
            1 => ChromaFormat::YUV420,
            2 => ChromaFormat::YUV422,
            3 => ChromaFormat::YUV444,
            _ => ChromaFormat::Invalid(chroma_format_idc)
        }
    }
}

// _Profile Indication_ value
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ProfileIdc(u8);
impl ProfileIdc {
    pub fn has_chroma_info(self) -> bool {
        match self.0 {
            100 | 110 | 122 | 244 | 44 | 83 | 86 => true,
            _ => false,
        }
    }
}
impl From<u8> for ProfileIdc {
    fn from(v: u8) -> Self {
        ProfileIdc(v)
    }
}
impl From<ProfileIdc> for u8 {
    fn from(v: ProfileIdc) -> Self { v.0 }
}

pub struct ScalingList {
    // TODO
}
impl ScalingList {
    pub fn read(r: &mut RbspBitReader<'_>, size: u8) -> Result<ScalingList,bitreader::BitReaderError> {
        let mut scaling_list = vec!();
        let mut last_scale = 8;
        let mut next_scale = 8;
        let mut _use_default_scaling_matrix_flag = false;
        for j in 0..size {
            if next_scale != 0 {
                let delta_scale = r.read_se()?;
                next_scale = (last_scale + delta_scale + 256) % 256;
                _use_default_scaling_matrix_flag = j == 0 && next_scale == 0;
            }
            let new_value = if next_scale == 0 { last_scale } else { next_scale };
            scaling_list.push(new_value);
            last_scale = new_value;
        }
        Ok(ScalingList { })
    }
}
#[derive(Debug, Clone)]
pub struct SeqScalingMatrix {
    // TODO
}
impl Default for SeqScalingMatrix {
    fn default() -> Self {
        SeqScalingMatrix { }
    }
}
impl SeqScalingMatrix {
    fn read(r: &mut RbspBitReader<'_>, chroma_format_idc: u32) -> Result<SeqScalingMatrix,bitreader::BitReaderError> {
        let mut scaling_list4x4 = vec!();
        let mut scaling_list8x8 = vec!();

        let count = if chroma_format_idc == 3 { 12 } else { 8 };
        for i in 0..count {
            let seq_scaling_list_present_flag = r.read_bool()?;
            if seq_scaling_list_present_flag {
                if i < 6 {
                    scaling_list4x4.push(ScalingList::read(r, 16)?);
                } else {
                    scaling_list8x8.push(ScalingList::read(r, 64)?);
                }
            }
        }
        Ok(SeqScalingMatrix { })
    }
}

#[derive(Debug, Clone)]
pub struct ChromaInfo {
    pub chroma_format: ChromaFormat,
    pub separate_colour_plane_flag: bool,
    pub bit_depth_luma_minus8: u8,
    pub bit_depth_chroma_minus8: u8,
    pub qpprime_y_zero_transform_bypass_flag: bool,
    pub scaling_matrix: SeqScalingMatrix,
}
impl ChromaInfo {
    pub fn read(r: &mut RbspBitReader<'_>, profile_idc: ProfileIdc) -> Result<ChromaInfo, SpsError> {
        if profile_idc.has_chroma_info() {
            let chroma_format_idc = r.read_ue()?;
            Ok(ChromaInfo {
                chroma_format: ChromaFormat::from_chroma_format_idc(chroma_format_idc),
                separate_colour_plane_flag: if chroma_format_idc == 3 { r.read_bool()? } else { false },
                bit_depth_luma_minus8: Self::read_bit_depth_minus8(r)?,
                bit_depth_chroma_minus8: Self::read_bit_depth_minus8(r)?,
                qpprime_y_zero_transform_bypass_flag: r.read_bool()?,
                scaling_matrix: Self::read_scaling_matrix(r, chroma_format_idc)?
            })
        } else {
            Ok(ChromaInfo {
                chroma_format: ChromaFormat::YUV420,
                separate_colour_plane_flag: false,
                bit_depth_luma_minus8: 0,
                bit_depth_chroma_minus8: 0,
                qpprime_y_zero_transform_bypass_flag: false,
                scaling_matrix: SeqScalingMatrix::default(),
            })
        }
    }
    fn read_bit_depth_minus8(r: &mut RbspBitReader<'_>) -> Result<u8, SpsError> {
        let value = r.read_ue()?;
        if value > 6 {
            Err(SpsError::BitDepthOutOfRange(value))
        } else {
            Ok(value as u8)
        }
    }
    fn read_scaling_matrix(r: &mut RbspBitReader<'_>, chroma_format_idc: u32) -> Result<SeqScalingMatrix, SpsError> {
        let scaling_matrix_present_flag = r.read_bool()?;
        if scaling_matrix_present_flag {
            SeqScalingMatrix::read(r, chroma_format_idc).map_err(|e| e.into())
        } else {
            Ok(SeqScalingMatrix::default())
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum PicOrderCntError {
    InvalidPicOrderCountType(u32),
    ReaderError(bitreader::BitReaderError),
    /// log2_max_pic_order_cnt_lsb_minus4 must be between 0 and 12
    Log2MaxPicOrderCntLsbMinus4OutOfRange(u32),
}

impl From<bitreader::BitReaderError> for PicOrderCntError {
    fn from(e: bitreader::BitReaderError) -> Self {
        PicOrderCntError::ReaderError(e)
    }
}

#[derive(Debug, Clone)]
pub enum PicOrderCntType {
    TypeZero {
        log2_max_pic_order_cnt_lsb_minus4: u8
    },
    TypeOne {
        delta_pic_order_always_zero_flag: bool,
        offset_for_non_ref_pic: i32,
        offset_for_top_to_bottom_field: i32,
        offsets_for_ref_frame: Vec<i32>
    },
    TypeTwo
}
impl PicOrderCntType {
    fn read(r: &mut RbspBitReader<'_>) -> Result<PicOrderCntType, PicOrderCntError> {
        let pic_order_cnt_type = r.read_ue()?;
        match pic_order_cnt_type {
            0 => {
                Ok(PicOrderCntType::TypeZero {
                    log2_max_pic_order_cnt_lsb_minus4: Self::read_log2_max_pic_order_cnt_lsb_minus4(r)?
                })
            },
            1 => {
                Ok(PicOrderCntType::TypeOne {
                    delta_pic_order_always_zero_flag: r.read_bool()?,
                    offset_for_non_ref_pic: r.read_se()?,
                    offset_for_top_to_bottom_field: r.read_se()?,
                    offsets_for_ref_frame: Self::read_offsets_for_ref_frame(r)?,
                })
            },
            2 => {
                Ok(PicOrderCntType::TypeTwo)
            },
            _ => {
                Err(PicOrderCntError::InvalidPicOrderCountType(pic_order_cnt_type))
            }
        }
    }

    fn read_log2_max_pic_order_cnt_lsb_minus4(r: &mut RbspBitReader<'_>) -> Result<u8, PicOrderCntError> {
        let val = r.read_ue()?;
        if val > 12 {
            Err(PicOrderCntError::Log2MaxPicOrderCntLsbMinus4OutOfRange(val))
        } else {
            Ok(val as u8)
        }
    }

    fn read_offsets_for_ref_frame(r: &mut RbspBitReader<'_>) -> Result<Vec<i32>, PicOrderCntError> {
        let num_ref_frames_in_pic_order_cnt_cycle = r.read_ue()?;
        let mut offsets = Vec::with_capacity(num_ref_frames_in_pic_order_cnt_cycle as usize);
        for _ in 0..num_ref_frames_in_pic_order_cnt_cycle {
            offsets.push(r.read_se()?);
        }
        Ok(offsets)
    }
}

#[derive(Debug, Clone)]
pub enum FrameMbsFlags {
    Frames,
    Fields {
        mb_adaptive_frame_field_flag: bool,
    }
}
impl FrameMbsFlags {
    fn read(r: &mut RbspBitReader<'_>) -> Result<FrameMbsFlags, bitreader::BitReaderError> {
        let frame_mbs_only_flag = r.read_bool()?;
        if frame_mbs_only_flag {
            Ok(FrameMbsFlags::Frames)
        } else {
            Ok(FrameMbsFlags::Fields {
                mb_adaptive_frame_field_flag: r.read_bool()?
            })
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FrameCropping {
    pub left_offset: u32,
    pub right_offset: u32,
    pub top_offset: u32,
    pub bottom_offset: u32,
}
impl FrameCropping {
    fn read(r: &mut RbspBitReader<'_>) -> Result<Option<FrameCropping>,bitreader::BitReaderError> {
        let frame_cropping_flag = r.read_bool()?;
        Ok(if frame_cropping_flag {
            Some(FrameCropping {
                left_offset: r.read_ue()?,
                right_offset: r.read_ue()?,
                top_offset: r.read_ue()?,
                bottom_offset: r.read_ue()?,
            })
        } else {
            None
        })
    }
}

#[derive(Debug, Clone)]
pub enum AspectRatioInfo {
    Unspecified,
    Ratio1_1,
    Ratio12_11,
    Ratio10_11,
    Ratio16_11,
    Ratio40_33,
    Ratio24_11,
    Ratio20_11,
    Ratio32_11,
    Ratio80_33,
    Ratio18_11,
    Ratio15_11,
    Ratio64_33,
    Ratio160_99,
    Ratio4_3,
    Ratio3_2,
    Ratio2_1,
    Reserved(u8),
    Extended(u16,u16),

}
impl AspectRatioInfo {
    fn read(r: &mut RbspBitReader<'_>) -> Result<Option<AspectRatioInfo>,bitreader::BitReaderError> {
        let aspect_ratio_info_present_flag = r.read_bool()?;
        Ok(if aspect_ratio_info_present_flag {
            let aspect_ratio_idc = r.read_u8(8)?;
            Some(match aspect_ratio_idc {
                0 => AspectRatioInfo::Unspecified,
                1 => AspectRatioInfo::Ratio1_1,
                2 => AspectRatioInfo::Ratio12_11,
                3 => AspectRatioInfo::Ratio10_11,
                4 => AspectRatioInfo::Ratio16_11,
                5 => AspectRatioInfo::Ratio40_33,
                6 => AspectRatioInfo::Ratio24_11,
                7 => AspectRatioInfo::Ratio20_11,
                8 => AspectRatioInfo::Ratio32_11,
                9 => AspectRatioInfo::Ratio80_33,
                10 => AspectRatioInfo::Ratio18_11,
                11 => AspectRatioInfo::Ratio15_11,
                12 => AspectRatioInfo::Ratio64_33,
                13 => AspectRatioInfo::Ratio160_99,
                14 => AspectRatioInfo::Ratio4_3,
                15 => AspectRatioInfo::Ratio3_2,
                16 => AspectRatioInfo::Ratio2_1,
                255 => AspectRatioInfo::Extended(r.read_u16(16)?, r.read_u16(16)?),
                _ => AspectRatioInfo::Reserved(aspect_ratio_idc),
            })
        } else {
            None
        })
    }

    /// Returns the aspect ratio as `(width, height)`, if specified.
    pub fn get(self) -> Option<(u16, u16)> {
        match self {
            AspectRatioInfo::Unspecified => None,
            AspectRatioInfo::Ratio1_1 => Some((1, 1)),
            AspectRatioInfo::Ratio12_11 => Some((12, 11)),
            AspectRatioInfo::Ratio10_11 => Some((10, 11)),
            AspectRatioInfo::Ratio16_11 => Some((16, 11)),
            AspectRatioInfo::Ratio40_33 => Some((40, 33)),
            AspectRatioInfo::Ratio24_11 => Some((24, 11)),
            AspectRatioInfo::Ratio20_11 => Some((20, 11)),
            AspectRatioInfo::Ratio32_11 => Some((32, 11)),
            AspectRatioInfo::Ratio80_33 => Some((80, 33)),
            AspectRatioInfo::Ratio18_11 => Some((18, 11)),
            AspectRatioInfo::Ratio15_11 => Some((15, 11)),
            AspectRatioInfo::Ratio64_33 => Some((64, 33)),
            AspectRatioInfo::Ratio160_99 => Some((160, 99)),
            AspectRatioInfo::Ratio4_3 => Some((4, 3)),
            AspectRatioInfo::Ratio3_2 => Some((3, 2)),
            AspectRatioInfo::Ratio2_1 => Some((2, 1)),
            AspectRatioInfo::Reserved(_) => None,
            AspectRatioInfo::Extended(width, height) => {
                // ISO/IEC 14496-10 section E.2.1: "When ... sar_width is equal to 0 or sar_height
                // is equal to 0, the sample aspect ratio shall be considered unspecified by this
                // Recommendation | International Standard."
                if width == 0 || height == 0 {
                    None
                } else {
                    Some((width, height))
                }
            },
        }
    }
}

#[derive(Debug, Clone)]
pub enum OverscanAppropriate {
    Unspecified,
    Appropriate,
    Inappropriate,
}
impl OverscanAppropriate {
    fn read(r: &mut RbspBitReader<'_>) -> Result<OverscanAppropriate,bitreader::BitReaderError> {
        let overscan_info_present_flag = r.read_bool()?;
        Ok(if overscan_info_present_flag {
            let overscan_appropriate_flag = r.read_bool()?;
            if overscan_appropriate_flag {
                OverscanAppropriate::Appropriate
            } else {
                OverscanAppropriate::Inappropriate
            }
        } else {
            OverscanAppropriate::Unspecified
        })
    }
}

#[derive(Debug, Clone)]
pub enum VideoFormat {
    Component,
    PAL,
    NTSC,
    SECAM,
    MAC,
    Unspecified,
    Reserved(u8),
}
impl VideoFormat {
    fn from(video_format: u8) -> VideoFormat {
        match video_format {
            0 => VideoFormat::Component,
            1 => VideoFormat::PAL,
            2 => VideoFormat::NTSC,
            3 => VideoFormat::SECAM,
            4 => VideoFormat::MAC,
            5 => VideoFormat::Unspecified,
            6|7 => VideoFormat::Reserved(video_format),
            _ => panic!("unsupported video_format value {}", video_format),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ColourDescription {
    colour_primaries: u8,
    transfer_characteristics: u8,
    matrix_coefficients: u8,
}
impl ColourDescription {
    fn read(r: &mut RbspBitReader<'_>) -> Result<Option<ColourDescription>,bitreader::BitReaderError> {
        let colour_description_present_flag = r.read_bool()?;
        Ok(if colour_description_present_flag {
            Some(ColourDescription {
                colour_primaries: r.read_u8(8)?,
                transfer_characteristics: r.read_u8(8)?,
                matrix_coefficients: r.read_u8(8)?,
            })
        } else {
            None
        })
    }
}

#[derive(Debug, Clone)]
pub struct VideoSignalType {
    video_format: VideoFormat,
    video_full_range_flag: bool,
    colour_description: Option<ColourDescription>,
}
impl VideoSignalType {
    fn read(r: &mut RbspBitReader<'_>) -> Result<Option<VideoSignalType>,bitreader::BitReaderError> {
        let video_signal_type_present_flag = r.read_bool()?;
        Ok(if video_signal_type_present_flag {
            Some(VideoSignalType {
                video_format: VideoFormat::from(r.read_u8(3)?),
                video_full_range_flag: r.read_bool()?,
                colour_description: ColourDescription::read(r)?,
            })
        } else {
            None
        })
    }
}

#[derive(Debug, Clone)]
pub struct ChromaLocInfo {
    chroma_sample_loc_type_top_field: u32,
    chroma_sample_loc_type_bottom_field: u32,
}
impl ChromaLocInfo {
    fn read(r: &mut RbspBitReader<'_>) -> Result<Option<ChromaLocInfo>,bitreader::BitReaderError> {
        let chroma_loc_info_present_flag = r.read_bool()?;
        Ok(if chroma_loc_info_present_flag {
            Some(ChromaLocInfo {
                chroma_sample_loc_type_top_field: r.read_ue()?,
                chroma_sample_loc_type_bottom_field: r.read_ue()?,
            })
        } else {
            None
        })
    }
}

#[derive(Debug, Clone)]
pub struct TimingInfo {
    num_units_in_tick: u32,
    time_scale: u32,
    fixed_frame_rate_flag: bool,
}
impl TimingInfo {
    fn read(r: &mut RbspBitReader<'_>) -> Result<Option<TimingInfo>,bitreader::BitReaderError> {
        let timing_info_present_flag = r.read_bool()?;
        Ok(if timing_info_present_flag {
            Some(TimingInfo {
                num_units_in_tick: r.read_u32(32)?,
                time_scale: r.read_u32(32)?,
                fixed_frame_rate_flag: r.read_bool()?,
            })
        } else {
            None
        })
    }
}

#[derive(Debug, Clone)]
pub struct CpbSpec {
    bit_rate_value_minus1: u32,
    cpb_size_value_minus1: u32,
    cbr_flag: bool,
}
impl CpbSpec {
    fn read(r: &mut RbspBitReader<'_>) -> Result<CpbSpec,RbspBitReaderError> {
        Ok(CpbSpec {
            bit_rate_value_minus1: r.read_ue_named("bit_rate_value_minus1")?,
            cpb_size_value_minus1: r.read_ue_named("cpb_size_value_minus1")?,
            cbr_flag: r.read_bool_named("cbr_flag")?,
        })
    }
}


#[derive(Debug, Clone)]
pub struct HrdParameters {
    pub bit_rate_scale: u8,
    pub cpb_size_scale: u8,
    pub cpb_specs: Vec<CpbSpec>,
    pub initial_cpb_removal_delay_length_minus1: u8,
    pub cpb_removal_delay_length_minus1: u8,
    pub dpb_output_delay_length_minus1: u8,
    pub time_offset_length: u8,
}
impl HrdParameters {
    fn read(r: &mut RbspBitReader<'_>, hrd_parameters_present: &mut bool) -> Result<Option<HrdParameters>,RbspBitReaderError> {
        let hrd_parameters_present_flag = r.read_bool_named("hrd_parameters_present_flag")?;
        *hrd_parameters_present |= hrd_parameters_present_flag;
        Ok(if hrd_parameters_present_flag {
            let cpb_cnt_minus1 = r.read_ue_named("cpb_cnt_minus1")?;
            let cpb_cnt = cpb_cnt_minus1 + 1;
            Some(HrdParameters {
                bit_rate_scale: r.read_u8(4)?,
                cpb_size_scale: r.read_u8(4)?,
                cpb_specs: Self::read_cpb_specs(r, cpb_cnt)?,
                initial_cpb_removal_delay_length_minus1: r.read_u8(5)?,
                cpb_removal_delay_length_minus1: r.read_u8(5)?,
                dpb_output_delay_length_minus1: r.read_u8(5)?,
                time_offset_length: r.read_u8(5)?,
            })
        } else {
            None
        })
    }
    fn read_cpb_specs(r: &mut RbspBitReader<'_>, cpb_cnt: u32) -> Result<Vec<CpbSpec>,RbspBitReaderError> {
        let mut cpb_specs = Vec::with_capacity(cpb_cnt as usize);
        for _ in 0..cpb_cnt {
            cpb_specs.push(CpbSpec::read(r)?);
        }
        Ok(cpb_specs)
    }
}

#[derive(Debug, Clone)]
pub struct BitstreamRestrictions {
    motion_vectors_over_pic_boundaries_flag: bool,
    max_bytes_per_pic_denom: u32,
    max_bits_per_mb_denom: u32,
    log2_max_mv_length_horizontal: u32,
    log2_max_mv_length_vertical: u32,
    max_num_reorder_frames: u32,
    max_dec_frame_buffering: u32,
}
impl BitstreamRestrictions {
    fn read(r: &mut RbspBitReader<'_>) -> Result<Option<BitstreamRestrictions>,RbspBitReaderError> {
        let bitstream_restriction_flag = r.read_bool()?;
        Ok(if bitstream_restriction_flag {
            Some(BitstreamRestrictions {
                motion_vectors_over_pic_boundaries_flag: r.read_bool_named("motion_vectors_over_pic_boundaries_flag")?,
                max_bytes_per_pic_denom: r.read_ue_named("max_bytes_per_pic_denom")?,
                max_bits_per_mb_denom: r.read_ue_named("max_bits_per_mb_denom")?,
                log2_max_mv_length_horizontal: r.read_ue_named("log2_max_mv_length_horizontal")?,
                log2_max_mv_length_vertical: r.read_ue_named("log2_max_mv_length_vertical")?,
                max_num_reorder_frames: r.read_ue_named("max_num_reorder_frames")?,
                max_dec_frame_buffering: r.read_ue_named("max_dec_frame_buffering")?,
            })
        } else {
            None
        })
    }
}

#[derive(Debug, Clone)]
pub struct VuiParameters {
    pub aspect_ratio_info: Option<AspectRatioInfo>,
    pub overscan_appropriate: OverscanAppropriate,
    pub video_signal_type: Option<VideoSignalType>,
    pub chroma_loc_info: Option<ChromaLocInfo>,
    pub timing_info: Option<TimingInfo>,
    pub nal_hrd_parameters: Option<HrdParameters>,
    pub vcl_hrd_parameters: Option<HrdParameters>,
    pub low_delay_hrd_flag: Option<bool>,
    pub pic_struct_present_flag: bool,
    pub bitstream_restrictions: Option<BitstreamRestrictions>,
}
impl VuiParameters {
    fn read(r: &mut RbspBitReader<'_>) -> Result<Option<VuiParameters>,RbspBitReaderError> {
        let vui_parameters_present_flag = r.read_bool()?;
        Ok(if vui_parameters_present_flag {
            let mut hrd_parameters_present = false;
            Some(VuiParameters {
                aspect_ratio_info: AspectRatioInfo::read(r)?,
                overscan_appropriate: OverscanAppropriate::read(r)?,
                video_signal_type: VideoSignalType::read(r)?,
                chroma_loc_info: ChromaLocInfo::read(r)?,
                timing_info: TimingInfo::read(r)?,
                nal_hrd_parameters: HrdParameters::read(r, &mut hrd_parameters_present)?,
                vcl_hrd_parameters: HrdParameters::read(r, &mut hrd_parameters_present)?,
                low_delay_hrd_flag: if hrd_parameters_present { Some(r.read_bool_named("low_delay_hrd_flag")?) } else { None },
                pic_struct_present_flag: r.read_bool_named("pic_struct_present_flag")?,
                bitstream_restrictions: BitstreamRestrictions::read(r)?,
            })
        } else {
            None
        })
    }
}

#[derive(Debug, Clone)]
pub struct SeqParameterSet {
    pub profile_idc: ProfileIdc,
    pub constraint_flags: ConstraintFlags,
    pub level_idc: u8,
    pub seq_parameter_set_id: ParamSetId,
    pub chroma_info: ChromaInfo,
    pub log2_max_frame_num_minus4: u8,
    pub pic_order_cnt: PicOrderCntType,
    pub max_num_ref_frames: u32,
    pub gaps_in_frame_num_value_allowed_flag: bool,
    pub pic_width_in_mbs_minus1: u32,
    pub pic_height_in_map_units_minus1: u32,
    pub frame_mbs_flags: FrameMbsFlags,
    pub direct_8x8_inference_flag: bool,
    pub frame_cropping: Option<FrameCropping>,
    pub vui_parameters: Option<VuiParameters>,
}
impl SeqParameterSet {
    pub fn from_bytes(buf: &[u8]) -> Result<SeqParameterSet, SpsError> {
        let mut r = RbspBitReader::new(buf);
        let profile_idc = r.read_u8(8)?.into();
        let sps = SeqParameterSet {
            profile_idc,
            constraint_flags: r.read_u8(8)?.into(),
            level_idc: r.read_u8(8)?,
            seq_parameter_set_id: ParamSetId::from_u32(r.read_ue_named("seq_parameter_set_id")?).map_err(SpsError::BadSeqParamSetId)?,
            chroma_info: ChromaInfo::read(&mut r, profile_idc)?,
            log2_max_frame_num_minus4: Self::read_log2_max_frame_num_minus4(&mut r)?,
            pic_order_cnt: PicOrderCntType::read(&mut r).map_err(SpsError::PicOrderCnt)?,
            max_num_ref_frames: r.read_ue()?,
            gaps_in_frame_num_value_allowed_flag: r.read_bool()?,
            pic_width_in_mbs_minus1: r.read_ue()?,
            pic_height_in_map_units_minus1: r.read_ue()?,
            frame_mbs_flags: FrameMbsFlags::read(&mut r)?,
            direct_8x8_inference_flag: r.read_bool()?,
            frame_cropping: FrameCropping::read(&mut r)?,
            vui_parameters: VuiParameters::read(&mut r)?,
        };
        Ok(sps)
    }

    fn read_log2_max_frame_num_minus4(r: &mut RbspBitReader<'_>) -> Result<u8, SpsError> {
        let val = r.read_ue()?;
        if val > 12 {
            Err(SpsError::Log2MaxFrameNumMinus4OutOfRange(val))
        } else {
            Ok(val as u8)
        }
    }

    pub fn profile(&self) -> Profile {
        Profile::from_profile_idc(self.profile_idc)
    }

    pub fn level(&self) -> Level {
        Level::from_constraint_flags_and_level_idc(self.constraint_flags, self.level_idc)
    }
    /// returned value will be in the range 4 to 16 inclusive
    pub fn log2_max_frame_num(&self) -> u8 {
        self.log2_max_frame_num_minus4 + 4
    }

    /// Helper to calculate the pixel-dimensions of the video image specified by this SPS, taking
    /// into account sample-format, interlacing and cropping.
    pub fn pixel_dimensions(&self) -> Result<(u32, u32), SpsError> {
        let width = (self.pic_width_in_mbs_minus1 + 1) * 16;
        let mul = match self.frame_mbs_flags {
            FrameMbsFlags::Fields { .. } => 2,
            FrameMbsFlags::Frames => 1,
        };
        let vsub = if self.chroma_info.chroma_format == ChromaFormat::YUV420 {
            1
        } else {
            0
        };
        let hsub = if self.chroma_info.chroma_format == ChromaFormat::YUV420
            || self.chroma_info.chroma_format == ChromaFormat::YUV422
        {
            1
        } else {
            0
        };

        let step_x = 1 << hsub;
        let step_y = mul << vsub;

        let height = (self.pic_height_in_map_units_minus1 + 1)
            .checked_mul(mul * 16)
            .ok_or_else(|| SpsError::FieldValueTooLarge { name:"pic_height_in_map_units_minus1", value: self.pic_height_in_map_units_minus1 })?;
        if let Some(ref crop) = self.frame_cropping {
            let left_offset = crop.left_offset.checked_mul(step_x)
                .ok_or_else(|| SpsError::FieldValueTooLarge { name:"left_offset", value: crop.left_offset })?;
            let right_offset = crop.right_offset.checked_mul(step_x)
                .ok_or_else(|| SpsError::FieldValueTooLarge { name:"right_offset", value: crop.right_offset })?;
            let top_offset = crop.top_offset.checked_mul(step_y)
                .ok_or_else(|| SpsError::FieldValueTooLarge { name:"top_offset", value: crop.top_offset })?;
            let bottom_offset = crop.bottom_offset.checked_mul(step_y)
                .ok_or_else(|| SpsError::FieldValueTooLarge { name:"bottom_offset", value: crop.bottom_offset })?;
            let width = width
                .checked_sub(left_offset)
                .and_then(|w| w.checked_sub(right_offset) );
            let height = height
                .checked_sub(top_offset)
                .and_then(|w| w.checked_sub(bottom_offset) );
            if let (Some(width), Some(height)) = (width, height) {
                Ok((width, height))
            } else {
                Err(SpsError::CroppingError(crop.clone()))
            }
        } else {
            Ok((width, height))
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use hex_literal::*;

    #[test]
    fn test_it() {
        let data = hex!(
           "64 00 0A AC 72 84 44 26 84 00 00
            00 04 00 00 00 CA 3C 48 96 11 80");
        let sps = SeqParameterSet::from_bytes(&data[..]).unwrap();
        println!("sps: {:#?}", sps);
        assert_eq!(100, sps.profile_idc.0);
        assert_eq!(0, sps.constraint_flags.reserved_zero_two_bits());
        assert_eq!(Ok((64, 64)), sps.pixel_dimensions());
    }

    #[test]
    fn test_dahua() {
        // From a Dahua IPC-HDW5231R-Z's sub stream, which is anamorphic.
        let data = hex!(
          "64 00 16 AC 1B 1A 80 B0 3D FF FF
           00 28 00 21 6E 0C 0C 0C 80 00 01
           F4 00 00 27 10 74 30 07 D0 00 07
           A1 25 DE 5C 68 60 0F A0 00 0F 42
           4B BC B8 50");
        let sps = SeqParameterSet::from_bytes(&data[..]).unwrap();
        println!("sps: {:#?}", sps);
        assert_eq!(sps.vui_parameters.unwrap().aspect_ratio_info.unwrap().get(), Some((40, 33)));
    }

    #[test]
    fn crop_removes_all_pixels() {
        let sps = SeqParameterSet {
            profile_idc: ProfileIdc(0),
            constraint_flags: ConstraintFlags(0),
            level_idc: 0,
            seq_parameter_set_id: ParamSetId::from_u32(0).unwrap(),
            chroma_info: ChromaInfo {
                chroma_format: ChromaFormat::Monochrome,
                separate_colour_plane_flag: false,
                bit_depth_luma_minus8: 0,
                bit_depth_chroma_minus8: 0,
                qpprime_y_zero_transform_bypass_flag: false,
                scaling_matrix: Default::default()
            },
            log2_max_frame_num_minus4: 0,
            pic_order_cnt: PicOrderCntType::TypeTwo,
            max_num_ref_frames: 0,
            frame_cropping: Some(FrameCropping {
                bottom_offset: 20,
                left_offset: 20,
                right_offset: 20,
                top_offset: 20
            }),
            pic_width_in_mbs_minus1: 1,
            pic_height_in_map_units_minus1: 1,
            frame_mbs_flags: FrameMbsFlags::Frames,
            gaps_in_frame_num_value_allowed_flag: false,
            direct_8x8_inference_flag: false,
            vui_parameters: None
        };
        /// should return Err, rather than assert due to integer underflow for example,
        let dim = sps.pixel_dimensions();
        assert!(matches!(dim, Err(SpsError::CroppingError(_))));
    }
}
