//! Rust implementations of various structs used by the OpenCL API.

use num::FromPrimitive;
use error::{Error as OclError, Result as OclResult};
use util;
use cl_h::{self, cl_mem};
use core::{Mem, MemObjectType, ImageChannelOrder, ImageChannelDataType, 
        ContextProperty, ContextInfoOrPropertiesPointerType as PropKind, PlatformId};


/// Context properties list.
///
/// [MINIMALLY TESTED]
///
/// TODO: Check for duplicate property assignments.
#[derive(Clone, Debug)]
pub struct ContextProperties(Vec<ContextProperty>);

impl ContextProperties {
    /// Returns an empty new list of context properties
    pub fn new() -> ContextProperties {
        ContextProperties(Vec::with_capacity(4))
    }

    /// Specifies a platform (builder-style).
    pub fn platform<P: Into<PlatformId>>(mut self, platform: P) -> ContextProperties {
        self.0.push(ContextProperty::Platform(platform.into()));
        self
    }

    /// Specifies whether the user is responsible for synchronization between
    /// OpenCL and other APIs (builder-style).
    pub fn interop_user_sync(mut self, sync: bool) -> ContextProperties {
        self.0.push(ContextProperty::InteropUserSync(sync));
        self
    }

    /// Pushes a `ContextProperty` onto this list of properties.
    pub fn and(mut self, prop: ContextProperty) -> ContextProperties {
        self.0.push(prop);
        self
    }

    /// Returns a platform id or none.
    pub fn get_platform(&self) -> Option<PlatformId> {
        let mut platform = None;

        for prop in self.0.iter() {
            if let &ContextProperty::Platform(ref plat) = prop {
                platform = Some(plat.clone());
            }
        }

        platform
    } 

    /// [UNTESTED: Not properly tested]
    /// Converts this list into a packed-byte representation as specified
    /// [here](https://www.khronos.org/registry/cl/sdk/1.2/docs/man/xhtml/clCreateContext.html).
    ///
    /// TODO: Evaluate cleaner ways to do this.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(128);

        unsafe { 
            // For each property:
            for prop in self.0.iter() {
                // Convert both the kind of property (a u32) and the value (variable type/size) 
                // into just a core byte vector (Vec<u8>):
                let (kind, val) = match prop {
                    &ContextProperty::Platform(ref platform_id_core) => (
                        util::into_bytes(PropKind::Platform as cl_h::cl_uint),
                        util::into_bytes(platform_id_core.as_ptr() as cl_h::cl_platform_id) 
                    ),
                    &ContextProperty::InteropUserSync(sync) => (
                        util::into_bytes(PropKind::InteropUserSync as cl_h::cl_uint),
                        util::into_bytes(sync as cl_h::cl_bool)
                    ),
                    _ => continue,
                };

                // Property Kind Enum:
                bytes.extend_from_slice(&kind);
                // 32 bits of padding:
                bytes.extend_from_slice(&util::into_bytes(0 as u32));
                // Value:
                bytes.extend_from_slice(&val);
                // 32 bits of padding:
                bytes.extend_from_slice(&util::into_bytes(0 as u32));
            }

            // Add a terminating 0:
            bytes.extend_from_slice(&util::into_bytes(0 as usize));
        }

        bytes.shrink_to_fit();
        bytes
    }
}

impl Into<Vec<ContextProperty>> for ContextProperties {
    fn into(self) -> Vec<ContextProperty> {
        self.0
    }
}

// pub enum ContextInfoOrPropertiesPointerType {
//     Platform = cl_h::CL_CONTEXT_PLATFORM as isize,
//     InteropUserSync = cl_h::CL_CONTEXT_INTEROP_USER_SYNC as isize,
// }

impl Into<Vec<u8>> for ContextProperties {
    fn into(self) -> Vec<u8> {
        self.to_bytes()
    }
}



/// Defines a buffer region for creating a sub-buffer.
///
/// ### Info (from [SDK](https://www.khronos.org/registry/cl/sdk/1.2/docs/man/xhtml/clCreateSubBuffer.html))
///
/// (origin, size) defines the offset and size in bytes in buffer.
///
/// If buffer is created with CL_MEM_USE_HOST_PTR, the host_ptr associated with
/// the buffer object returned is host_ptr + origin.
///
/// The buffer object returned references the data store allocated for buffer and
/// points to a specific region given by (origin, size) in this data store.
///
/// CL_INVALID_VALUE is returned in errcode_ret if the region specified by
/// (origin, size) is out of bounds in buffer.
///
/// CL_INVALID_BUFFER_SIZE if size is 0.
///
/// CL_MISALIGNED_SUB_BUFFER_OFFSET is returned in errcode_ret if there are no
/// devices in context associated with buffer for which the origin value is
/// aligned to the CL_DEVICE_MEM_BASE_ADDR_ALIGN value.
///
pub struct BufferRegion {
    pub origin: usize,
    pub size: usize,
}


/// Image format properties used by `Image`.
///
/// A structure that describes format properties of the image to be allocated. (from SDK)
///
/// # Examples (from SDK)
///
/// To specify a normalized unsigned 8-bit / channel RGBA image:
///    image_channel_order = CL_RGBA
///    image_channel_data_type = CL_UNORM_INT8
///
/// image_channel_data_type values of CL_UNORM_SHORT_565, CL_UNORM_SHORT_555 and CL_UNORM_INT_101010 are special cases of packed image formats where the channels of each element are packed into a single unsigned short or unsigned int. For these special packed image formats, the channels are normally packed with the first channel in the most significant bits of the bitfield, and successive channels occupying progressively less significant locations. For CL_UNORM_SHORT_565, R is in bits 15:11, G is in bits 10:5 and B is in bits 4:0. For CL_UNORM_SHORT_555, bit 15 is undefined, R is in bits 14:10, G in bits 9:5 and B in bits 4:0. For CL_UNORM_INT_101010, bits 31:30 are undefined, R is in bits 29:20, G in bits 19:10 and B in bits 9:0.
/// OpenCL implementations must maintain the minimum precision specified by the number of bits in image_channel_data_type. If the image format specified by image_channel_order, and image_channel_data_type cannot be supported by the OpenCL implementation, then the call to clCreateImage will return a NULL memory object.
///
#[derive(Debug, Clone)]
pub struct ImageFormat {
    pub channel_order: ImageChannelOrder,
    pub channel_data_type: ImageChannelDataType,
}

impl ImageFormat {
    pub fn new(order: ImageChannelOrder, data_type: ImageChannelDataType) -> ImageFormat {
        ImageFormat {
            channel_order: order,
            channel_data_type: data_type,
        }
    }

    pub fn new_rgba() -> ImageFormat {
        ImageFormat {
            channel_order: ImageChannelOrder::Rgba,
            channel_data_type: ImageChannelDataType::SnormInt8,
        }
    }

    pub fn from_raw(raw: cl_h::cl_image_format) -> OclResult<ImageFormat> {
        Ok(ImageFormat {
            channel_order: try!(ImageChannelOrder::from_u32(raw.image_channel_order)
                .ok_or(OclError::new("Error converting to 'ImageChannelOrder'."))),
            channel_data_type: try!(ImageChannelDataType::from_u32(raw.image_channel_data_type)
                .ok_or(OclError::new("Error converting to 'ImageChannelDataType'."))),
        })
    }

    pub fn list_from_raw(list_raw: Vec<cl_h::cl_image_format>) -> OclResult<Vec<ImageFormat>> {
        let mut result_list = Vec::with_capacity(list_raw.len());

        for clif in list_raw.into_iter() {
            result_list.push(try!(ImageFormat::from_raw(clif)));
        }

        Ok(result_list)
    }

    pub fn to_raw(&self) -> cl_h::cl_image_format {
        cl_h::cl_image_format {
            image_channel_order: self.channel_order as cl_h::cl_channel_order,
            image_channel_data_type: self.channel_data_type as cl_h::cl_channel_type,
        }
    }

    pub fn new_raw() -> cl_h::cl_image_format {
        cl_h::cl_image_format {
            image_channel_order: 0 as cl_h::cl_channel_order,
            image_channel_data_type: 0 as cl_h::cl_channel_type,
        }
    }

    /// Returns the size in bytes of a pixel using the format specified by this
    /// `ImageFormat`.
    ///
    /// TODO: Add a special case for Depth & DepthStencil
    /// (https://www.khronos.org/registry/cl/sdk/2.0/docs/man/xhtml/cl_khr_gl_depth_images.html).
    /// 
    /// TODO: Validate combinations.
    /// TODO: Use `core::get_image_info` to check these with a test.
    ///
    pub fn pixel_bytes(&self) -> usize {
        let channel_count = match self.channel_order {
            ImageChannelOrder::R => 1,
            ImageChannelOrder::A => 1,
            ImageChannelOrder::Rg => 2,
            ImageChannelOrder::Ra => 2,
            // This format can only be used if channel data type = CL_UNORM_SHORT_565, CL_UNORM_SHORT_555 or CL_UNORM_INT101010:
            ImageChannelOrder::Rgb => 1,
            ImageChannelOrder::Rgba => 4,
            // This format can only be used if channel data type = CL_UNORM_INT8, CL_SNORM_INT8, CL_SIGNED_INT8 or CL_UNSIGNED_INT8:
            ImageChannelOrder::Bgra => 4,
            // This format can only be used if channel data type = CL_UNORM_INT8, CL_SNORM_INT8, CL_SIGNED_INT8 or CL_UNSIGNED_INT8:
            ImageChannelOrder::Argb => 4,
            // This format can only be used if channel data type = CL_UNORM_INT8, CL_UNORM_INT16, CL_SNORM_INT8, CL_SNORM_INT16, CL_HALF_FLOAT, or CL_FLOAT:
            ImageChannelOrder::Intensity => 4,
            // This format can only be used if channel data type = CL_UNORM_INT8, CL_UNORM_INT16, CL_SNORM_INT8, CL_SNORM_INT16, CL_HALF_FLOAT, or CL_FLOAT:
            ImageChannelOrder::Luminance => 4,
            ImageChannelOrder::Rx => 2,
            ImageChannelOrder::Rgx => 4,
            // This format can only be used if channel data type = CL_UNORM_SHORT_565, CL_UNORM_SHORT_555 or CL_UNORM_INT101010:
            ImageChannelOrder::Rgbx => 4,
            // Depth => 1,
            // DepthStencil => 1,
            _ => 0,
        };

        let channel_size = match self.channel_data_type {
            // Each channel component is a normalized signed 8-bit integer value:
            ImageChannelDataType::SnormInt8 => 1,
            // Each channel component is a normalized signed 16-bit integer value:
            ImageChannelDataType::SnormInt16 => 2,
            // Each channel component is a normalized unsigned 8-bit integer value:
            ImageChannelDataType::UnormInt8 => 1,
            // Each channel component is a normalized unsigned 16-bit integer value:
            ImageChannelDataType::UnormInt16 => 2,
            // Represents a normalized 5-6-5 3-channel RGB image. The channel order must be CL_RGB or CL_RGBx:
            ImageChannelDataType::UnormShort565 => 2,
            // Represents a normalized x-5-5-5 4-channel xRGB image. The channel order must be CL_RGB or CL_RGBx:
            ImageChannelDataType::UnormShort555 => 2,
            // Represents a normalized x-10-10-10 4-channel xRGB image. The channel order must be CL_RGB or CL_RGBx:
            ImageChannelDataType::UnormInt101010 => 4,
            // Each channel component is an unnormalized signed 8-bit integer value:
            ImageChannelDataType::SignedInt8 => 1,
            // Each channel component is an unnormalized signed 16-bit integer value:
            ImageChannelDataType::SignedInt16 => 2,
            // Each channel component is an unnormalized signed 32-bit integer value:
            ImageChannelDataType::SignedInt32 => 4,
            // Each channel component is an unnormalized unsigned 8-bit integer value:
            ImageChannelDataType::UnsignedInt8 => 1,
            // Each channel component is an unnormalized unsigned 16-bit integer value:
            ImageChannelDataType::UnsignedInt16 => 2,
            // Each channel component is an unnormalized unsigned 32-bit integer value:
            ImageChannelDataType::UnsignedInt32 => 4,
            // Each channel component is a 16-bit half-float value:
            ImageChannelDataType::HalfFloat => 2,
            // Each channel component is a single precision floating-point value:
            ImageChannelDataType::Float => 4,
            // Each channel component is a normalized unsigned 24-bit integer value:
            // UnormInt24 => 3,
            _ => 0
        };

        channel_count * channel_size
    }
}


/// An image descriptor use in the creation of `Image`.
///
/// image_type
/// Describes the image type and must be either CL_MEM_OBJECT_IMAGE1D, CL_MEM_OBJECT_IMAGE1D_BUFFER, CL_MEM_OBJECT_IMAGE1D_ARRAY, CL_MEM_OBJECT_IMAGE2D, CL_MEM_OBJECT_IMAGE2D_ARRAY, or CL_MEM_OBJECT_IMAGE3D.
///
/// image_width
/// The width of the image in pixels. For a 2D image and image array, the image width must be ≤ CL_DEVICE_IMAGE2D_MAX_WIDTH. For a 3D image, the image width must be ≤ CL_DEVICE_IMAGE3D_MAX_WIDTH. For a 1D image buffer, the image width must be ≤ CL_DEVICE_IMAGE_MAX_BUFFER_SIZE. For a 1D image and 1D image array, the image width must be ≤ CL_DEVICE_IMAGE2D_MAX_WIDTH.
///
/// image_height
/// The height of the image in pixels. This is only used if the image is a 2D, 3D or 2D image array. For a 2D image or image array, the image height must be ≤ CL_DEVICE_IMAGE2D_MAX_HEIGHT. For a 3D image, the image height must be ≤ CL_DEVICE_IMAGE3D_MAX_HEIGHT.
///
/// image_depth
/// The depth of the image in pixels. This is only used if the image is a 3D image and must be a value ≥ 1 and ≤ CL_DEVICE_IMAGE3D_MAX_DEPTH.
///
/// image_array_size
/// The number of images in the image array. This is only used if the image is a 1D or 2D image array. The values for image_array_size, if specified, must be a value ≥ 1 and ≤ CL_DEVICE_IMAGE_MAX_ARRAY_SIZE.
///
/// Note that reading and writing 2D image arrays from a kernel with image_array_size = 1 may be lower performance than 2D images.
///
/// image_row_pitch
/// The scan-line pitch in bytes. This must be 0 if host_ptr is NULL and can be either 0 or ≥ image_width * size of element in bytes if host_ptr is not NULL. If host_ptr is not NULL and image_row_pitch = 0, image_row_pitch is calculated as image_width * size of element in bytes. If image_row_pitch is not 0, it must be a multiple of the image element size in bytes.
///
/// image_slice_pitch
/// The size in bytes of each 2D slice in the 3D image or the size in bytes of each image in a 1D or 2D image array. This must be 0 if host_ptr is NULL. If host_ptr is not NULL, image_slice_pitch can be either 0 or ≥ image_row_pitch * image_height for a 2D image array or 3D image and can be either 0 or ≥ image_row_pitch for a 1D image array. If host_ptr is not NULL and image_slice_pitch = 0, image_slice_pitch is calculated as image_row_pitch * image_height for a 2D image array or 3D image and image_row_pitch for a 1D image array. If image_slice_pitch is not 0, it must be a multiple of the image_row_pitch.
///
/// num_mip_level, num_samples
/// Must be 0.
///
/// buffer
/// Refers to a valid buffer memory object if image_type is CL_MEM_OBJECT_IMAGE1D_BUFFER. Otherwise it must be NULL. For a 1D image buffer object, the image pixels are taken from the buffer object's data store. When the contents of a buffer object's data store are modified, those changes are reflected in the contents of the 1D image buffer object and vice-versa at corresponding sychronization points. The image_width * size of element in bytes must be ≤ size of buffer object data store.
///
/// Note
/// Concurrent reading from, writing to and copying between both a buffer object and 1D image buffer object associated with the buffer object is undefined. Only reading from both a buffer object and 1D image buffer object associated with the buffer object is defined.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ImageDescriptor {
    pub image_type: MemObjectType,
    pub image_width: usize,
    pub image_height: usize,
    pub image_depth: usize,
    pub image_array_size: usize,
    pub image_row_pitch: usize,
    pub image_slice_pitch: usize,
    num_mip_levels: u32,
    num_samples: u32,
    pub buffer: Option<Mem>,
}

impl ImageDescriptor {
    pub fn new(image_type: MemObjectType, width: usize, height: usize, depth: usize, 
                array_size: usize, row_pitch: usize, slc_pitch: usize, buffer: Option<Mem>,
                ) -> ImageDescriptor {
        ImageDescriptor {
            image_type: image_type,
            image_width: width,
            image_height: height,
            image_depth: depth,
            image_array_size: array_size,
            image_row_pitch: row_pitch,
            image_slice_pitch: slc_pitch,
            num_mip_levels: 0,
            num_samples: 0,
            buffer: buffer,
        }
    }

    pub fn to_raw(&self) -> cl_h::cl_image_desc {
        cl_h::cl_image_desc {
            image_type: self.image_type as u32,
            image_width: self.image_width,
            image_height: self.image_height,
            image_depth: self.image_depth,
            image_array_size: self.image_array_size,
            image_row_pitch: self.image_row_pitch,
            image_slice_pitch: self.image_slice_pitch,
            num_mip_levels: self.num_mip_levels,
            num_samples: self.num_mip_levels,
            buffer: match &self.buffer {
                    &Some(ref b) => unsafe { b.as_ptr() },
                    &None => 0 as cl_mem,
                },
        }
    }
}

