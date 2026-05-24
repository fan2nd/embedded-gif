#![no_std]
#![doc = include_str!("../README.md")]

pub use embedded_gif_macros::{include_complete_gif, include_raw_gif};
pub use embedded_graphics;

use embedded_graphics::{
    draw_target::DrawTarget,
    geometry::{OriginDimensions, Point},
    image::{GetPixel, ImageRaw},
    iterator::raw::RawDataSlice,
    pixelcolor::{
        raw::{BigEndian, ByteOrder},
        BinaryColor, PixelColor, Rgb565, Rgb888,
    },
    primitives::Rectangle,
    transform::Transform,
    Pixel,
};

pub type GifImage<C = Rgb888, BO = BigEndian> = ImageRaw<'static, C, BO>;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum DisposalMethod {
    Any,
    Keep,
    Background,
    Previous,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct CompleteGifFrame<C = Rgb888, BO = BigEndian>
where
    C: PixelColor + From<C::Raw> + 'static,
    BO: ByteOrder + 'static,
{
    image: GifImage<C, BO>,
    alpha_mask: &'static [u8],
    delay_centiseconds: u16,
}

impl<C, BO> CompleteGifFrame<C, BO>
where
    C: PixelColor + From<C::Raw> + 'static,
    BO: ByteOrder + 'static,
{
    pub const fn new(
        image: GifImage<C, BO>,
        alpha_mask: &'static [u8],
        delay_centiseconds: u16,
    ) -> Self {
        Self {
            image,
            alpha_mask,
            delay_centiseconds,
        }
    }

    pub const fn image(&self) -> &GifImage<C, BO> {
        &self.image
    }

    pub const fn alpha_mask(&self) -> &'static [u8] {
        self.alpha_mask
    }

    pub const fn delay_centiseconds(&self) -> u16 {
        self.delay_centiseconds
    }

    pub const fn delay_millis(&self) -> u32 {
        self.delay_centiseconds as u32 * 10
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct RawGifFrame<C = Rgb888, BO = BigEndian>
where
    C: PixelColor + From<C::Raw> + 'static,
    BO: ByteOrder + 'static,
{
    image: GifImage<C, BO>,
    alpha_mask: &'static [u8],
    top_left: Point,
    delay_centiseconds: u16,
    disposal_method: DisposalMethod,
}

impl<C, BO> RawGifFrame<C, BO>
where
    C: PixelColor + From<C::Raw> + 'static,
    BO: ByteOrder + 'static,
{
    pub const fn new(
        image: GifImage<C, BO>,
        alpha_mask: &'static [u8],
        top_left: Point,
        delay_centiseconds: u16,
        disposal_method: DisposalMethod,
    ) -> Self {
        Self {
            image,
            alpha_mask,
            top_left,
            delay_centiseconds,
            disposal_method,
        }
    }

    pub const fn image(&self) -> &GifImage<C, BO> {
        &self.image
    }

    pub const fn alpha_mask(&self) -> &'static [u8] {
        self.alpha_mask
    }

    pub const fn top_left(&self) -> Point {
        self.top_left
    }

    pub const fn delay_centiseconds(&self) -> u16 {
        self.delay_centiseconds
    }

    pub const fn delay_millis(&self) -> u32 {
        self.delay_centiseconds as u32 * 10
    }

    pub const fn disposal_method(&self) -> DisposalMethod {
        self.disposal_method
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct RawGifCompressedFrame<C = Rgb888>
where
    C: PixelColor + 'static,
{
    data: &'static [u8],
    size: embedded_graphics::geometry::Size,
    top_left: Point,
    delay_centiseconds: u16,
    disposal_method: DisposalMethod,
    pixel_color: core::marker::PhantomData<C>,
}

impl<C> RawGifCompressedFrame<C>
where
    C: PixelColor + 'static,
{
    pub const fn new(
        data: &'static [u8],
        size: embedded_graphics::geometry::Size,
        top_left: Point,
        delay_centiseconds: u16,
        disposal_method: DisposalMethod,
    ) -> Self {
        Self {
            data,
            size,
            top_left,
            delay_centiseconds,
            disposal_method,
            pixel_color: core::marker::PhantomData,
        }
    }

    pub const fn data(&self) -> &'static [u8] {
        self.data
    }

    pub const fn size(&self) -> embedded_graphics::geometry::Size {
        self.size
    }

    pub const fn top_left(&self) -> Point {
        self.top_left
    }

    pub const fn delay_centiseconds(&self) -> u16 {
        self.delay_centiseconds
    }

    pub const fn delay_millis(&self) -> u32 {
        self.delay_centiseconds as u32 * 10
    }

    pub const fn disposal_method(&self) -> DisposalMethod {
        self.disposal_method
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct CompleteGif<C = Rgb888, BO = BigEndian>
where
    C: PixelColor + From<C::Raw> + 'static,
    BO: ByteOrder + 'static,
{
    frames: &'static [CompleteGifFrame<C, BO>],
    index: usize,
    elapsed_millis: u32,
}

impl<C, BO> CompleteGif<C, BO>
where
    C: PixelColor + From<C::Raw> + 'static,
    BO: ByteOrder + 'static,
{
    pub const fn new(frames: &'static [CompleteGifFrame<C, BO>]) -> Self {
        Self {
            frames,
            index: 0,
            elapsed_millis: 0,
        }
    }

    pub const fn frames(&self) -> &'static [CompleteGifFrame<C, BO>] {
        self.frames
    }

    pub const fn len(&self) -> usize {
        self.frames.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.frames.is_empty()
    }

    pub const fn frame_index(&self) -> usize {
        self.index
    }

    pub fn current_frame(&self) -> Option<&CompleteGifFrame<C, BO>> {
        self.frames.get(self.index)
    }

    pub fn reset(&mut self) {
        self.index = 0;
        self.elapsed_millis = 0;
    }

    pub fn advance(&mut self) -> Option<&CompleteGifFrame<C, BO>> {
        if self.frames.is_empty() {
            return None;
        }

        self.index = (self.index + 1) % self.frames.len();
        self.elapsed_millis = 0;

        self.current_frame()
    }

    pub fn tick_millis(&mut self, millis: u32) -> bool {
        tick_frame(
            &mut self.index,
            &mut self.elapsed_millis,
            self.frames,
            millis,
        )
    }

    pub fn tick_centiseconds(&mut self, centiseconds: u32) -> bool {
        self.tick_millis(centiseconds.saturating_mul(10))
    }

    pub fn draw_current<D>(&self, target: &mut D, position: Point) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = C>,
        RawDataSlice<'static, C::Raw, BO>: IntoIterator<Item = C::Raw>,
    {
        if let Some(frame) = self.current_frame() {
            draw_masked_image(target, frame.image(), frame.alpha_mask(), position)
        } else {
            Ok(())
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct RawGif<C = Rgb888, BO = BigEndian>
where
    C: PixelColor + From<C::Raw> + 'static,
    BO: ByteOrder + 'static,
{
    frames: &'static [RawGifFrame<C, BO>],
    index: usize,
    elapsed_millis: u32,
    composited_index: Option<usize>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct RawCompressedGif<C = Rgb888>
where
    C: PixelColor + 'static,
{
    frames: &'static [RawGifCompressedFrame<C>],
    index: usize,
    elapsed_millis: u32,
    composited_index: Option<usize>,
}

impl<C, BO> RawGif<C, BO>
where
    C: PixelColor + From<C::Raw> + 'static,
    BO: ByteOrder + 'static,
{
    pub const fn new(frames: &'static [RawGifFrame<C, BO>]) -> Self {
        Self {
            frames,
            index: 0,
            elapsed_millis: 0,
            composited_index: None,
        }
    }

    pub const fn frames(&self) -> &'static [RawGifFrame<C, BO>] {
        self.frames
    }

    pub const fn len(&self) -> usize {
        self.frames.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.frames.is_empty()
    }

    pub const fn frame_index(&self) -> usize {
        self.index
    }

    pub fn current_frame(&self) -> Option<&RawGifFrame<C, BO>> {
        self.frames.get(self.index)
    }

    pub fn reset(&mut self) {
        self.index = 0;
        self.elapsed_millis = 0;
        self.composited_index = None;
    }

    pub fn advance(&mut self) -> Option<&RawGifFrame<C, BO>> {
        if self.frames.is_empty() {
            return None;
        }

        self.index = (self.index + 1) % self.frames.len();
        self.elapsed_millis = 0;

        self.current_frame()
    }

    pub fn tick_millis(&mut self, millis: u32) -> bool {
        tick_frame(
            &mut self.index,
            &mut self.elapsed_millis,
            self.frames,
            millis,
        )
    }

    pub fn tick_centiseconds(&mut self, centiseconds: u32) -> bool {
        self.tick_millis(centiseconds.saturating_mul(10))
    }

    pub fn draw_current_delta<D>(&self, target: &mut D, origin: Point) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = C>,
        RawDataSlice<'static, C::Raw, BO>: IntoIterator<Item = C::Raw>,
    {
        if let Some(frame) = self.current_frame() {
            Self::draw_frame(target, origin, frame)
        } else {
            Ok(())
        }
    }

    pub fn draw_current_composited<D>(
        &mut self,
        target: &mut D,
        origin: Point,
        background: C,
    ) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = C>,
        RawDataSlice<'static, C::Raw, BO>: IntoIterator<Item = C::Raw>,
    {
        if self.frames.is_empty() {
            return Ok(());
        }

        if self.composited_index == previous_index(self.index, self.frames.len()) {
            if let Some(previous) = self
                .composited_index
                .and_then(|index| self.frames.get(index))
            {
                Self::dispose_frame(target, origin, background, previous)?;
            }

            if let Some(frame) = self.current_frame() {
                Self::draw_frame(target, origin, frame)?;
            }

            self.composited_index = Some(self.index);
            return Ok(());
        }

        if let Some(bounds) = self.canvas_bounds() {
            target.fill_solid(&bounds.translate(origin), background)?;
        }

        for (frame_index, frame) in self.frames.iter().enumerate().take(self.index + 1) {
            if frame_index != self.index && frame.disposal_method() == DisposalMethod::Previous {
                continue;
            }

            Self::draw_frame(target, origin, frame)?;

            if frame_index != self.index {
                Self::dispose_frame(target, origin, background, frame)?;
            }
        }

        self.composited_index = Some(self.index);
        Ok(())
    }

    fn draw_frame<D>(
        target: &mut D,
        origin: Point,
        frame: &RawGifFrame<C, BO>,
    ) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = C>,
        RawDataSlice<'static, C::Raw, BO>: IntoIterator<Item = C::Raw>,
    {
        draw_masked_image(
            target,
            frame.image(),
            frame.alpha_mask(),
            origin + frame.top_left(),
        )
    }

    fn dispose_frame<D>(
        target: &mut D,
        origin: Point,
        background: C,
        frame: &RawGifFrame<C, BO>,
    ) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = C>,
    {
        match frame.disposal_method() {
            DisposalMethod::Any | DisposalMethod::Keep => Ok(()),
            DisposalMethod::Background => {
                target.fill_solid(&frame.bounding_box().translate(origin), background)
            }
            DisposalMethod::Previous => Ok(()),
        }
    }

    fn canvas_bounds(&self) -> Option<Rectangle> {
        canvas_bounds(self.frames)
    }
}

impl<C> RawCompressedGif<C>
where
    C: CompressedColor + PixelColor + 'static,
{
    pub const fn new(frames: &'static [RawGifCompressedFrame<C>]) -> Self {
        Self {
            frames,
            index: 0,
            elapsed_millis: 0,
            composited_index: None,
        }
    }

    pub const fn frames(&self) -> &'static [RawGifCompressedFrame<C>] {
        self.frames
    }

    pub const fn len(&self) -> usize {
        self.frames.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.frames.is_empty()
    }

    pub const fn frame_index(&self) -> usize {
        self.index
    }

    pub fn current_frame(&self) -> Option<&RawGifCompressedFrame<C>> {
        self.frames.get(self.index)
    }

    pub fn reset(&mut self) {
        self.index = 0;
        self.elapsed_millis = 0;
        self.composited_index = None;
    }

    pub fn advance(&mut self) -> Option<&RawGifCompressedFrame<C>> {
        if self.frames.is_empty() {
            return None;
        }

        self.index = (self.index + 1) % self.frames.len();
        self.elapsed_millis = 0;

        self.current_frame()
    }

    pub fn tick_millis(&mut self, millis: u32) -> bool {
        tick_frame(
            &mut self.index,
            &mut self.elapsed_millis,
            self.frames,
            millis,
        )
    }

    pub fn tick_centiseconds(&mut self, centiseconds: u32) -> bool {
        self.tick_millis(centiseconds.saturating_mul(10))
    }

    pub fn draw_current_delta<D>(&self, target: &mut D, origin: Point) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = C>,
    {
        if let Some(frame) = self.current_frame() {
            Self::draw_frame(target, origin, frame)
        } else {
            Ok(())
        }
    }

    pub fn draw_current_composited<D>(
        &mut self,
        target: &mut D,
        origin: Point,
        background: C,
    ) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = C>,
    {
        if self.frames.is_empty() {
            return Ok(());
        }

        if self.composited_index == previous_index(self.index, self.frames.len()) {
            if let Some(previous) = self
                .composited_index
                .and_then(|index| self.frames.get(index))
            {
                Self::dispose_frame(target, origin, background, previous)?;
            }

            if let Some(frame) = self.current_frame() {
                Self::draw_frame(target, origin, frame)?;
            }

            self.composited_index = Some(self.index);
            return Ok(());
        }

        if let Some(bounds) = self.canvas_bounds() {
            target.fill_solid(&bounds.translate(origin), background)?;
        }

        for (frame_index, frame) in self.frames.iter().enumerate().take(self.index + 1) {
            if frame_index != self.index && frame.disposal_method() == DisposalMethod::Previous {
                continue;
            }

            Self::draw_frame(target, origin, frame)?;

            if frame_index != self.index {
                Self::dispose_frame(target, origin, background, frame)?;
            }
        }

        self.composited_index = Some(self.index);
        Ok(())
    }

    fn draw_frame<D>(
        target: &mut D,
        origin: Point,
        frame: &RawGifCompressedFrame<C>,
    ) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = C>,
    {
        draw_compressed_frame(target, frame, origin)
    }

    fn dispose_frame<D>(
        target: &mut D,
        origin: Point,
        background: C,
        frame: &RawGifCompressedFrame<C>,
    ) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = C>,
    {
        match frame.disposal_method() {
            DisposalMethod::Any | DisposalMethod::Keep => Ok(()),
            DisposalMethod::Background => {
                target.fill_solid(&frame.bounding_box().translate(origin), background)
            }
            DisposalMethod::Previous => Ok(()),
        }
    }

    fn canvas_bounds(&self) -> Option<Rectangle> {
        canvas_bounds(self.frames)
    }
}

impl<C, BO> RawGifFrame<C, BO>
where
    C: PixelColor + From<C::Raw> + 'static,
    BO: ByteOrder + 'static,
{
    pub fn bounding_box(&self) -> Rectangle {
        Rectangle::new(self.top_left, self.image.size())
    }
}

impl<C> RawGifCompressedFrame<C>
where
    C: PixelColor + 'static,
{
    pub fn bounding_box(&self) -> Rectangle {
        Rectangle::new(self.top_left, self.size)
    }
}

trait GifFrameTiming {
    fn delay_millis(&self) -> u32;
}

impl<C, BO> GifFrameTiming for CompleteGifFrame<C, BO>
where
    C: PixelColor + From<C::Raw> + 'static,
    BO: ByteOrder + 'static,
{
    fn delay_millis(&self) -> u32 {
        self.delay_millis()
    }
}

impl<C, BO> GifFrameTiming for RawGifFrame<C, BO>
where
    C: PixelColor + From<C::Raw> + 'static,
    BO: ByteOrder + 'static,
{
    fn delay_millis(&self) -> u32 {
        self.delay_millis()
    }
}

impl<C> GifFrameTiming for RawGifCompressedFrame<C>
where
    C: PixelColor + 'static,
{
    fn delay_millis(&self) -> u32 {
        self.delay_millis()
    }
}

fn tick_frame<F>(
    index: &mut usize,
    elapsed_millis: &mut u32,
    frames: &'static [F],
    millis: u32,
) -> bool
where
    F: GifFrameTiming,
{
    if frames.len() <= 1 {
        return false;
    }

    *elapsed_millis = elapsed_millis.saturating_add(millis);

    let mut changed = false;
    while *elapsed_millis >= frames[*index].delay_millis().max(10) {
        *elapsed_millis -= frames[*index].delay_millis().max(10);
        *index = (*index + 1) % frames.len();
        changed = true;
    }

    changed
}

fn previous_index(index: usize, len: usize) -> Option<usize> {
    if len == 0 {
        None
    } else if index == 0 {
        Some(len - 1)
    } else {
        Some(index - 1)
    }
}

fn draw_masked_image<C, BO, D>(
    target: &mut D,
    image: &GifImage<C, BO>,
    alpha_mask: &[u8],
    position: Point,
) -> Result<(), D::Error>
where
    C: PixelColor + From<C::Raw> + 'static,
    BO: ByteOrder + 'static,
    D: DrawTarget<Color = C>,
    RawDataSlice<'static, C::Raw, BO>: IntoIterator<Item = C::Raw>,
{
    let size = image.size();
    let pixel_count = size.width as usize * size.height as usize;
    target.draw_iter((0..pixel_count).filter_map(|index| {
        if !mask_bit(alpha_mask, index) {
            return None;
        }

        let x = index as u32 % size.width;
        let y = index as u32 / size.width;

        if y >= size.height {
            return None;
        }

        let point = Point::new(x as i32, y as i32);
        image
            .pixel(point)
            .map(|color| Pixel(position + point, color))
    }))
}

fn mask_bit(mask: &[u8], index: usize) -> bool {
    mask.get(index / 8)
        .map(|byte| byte & (0x80 >> (index % 8)) != 0)
        .unwrap_or(false)
}

trait GifFrameBounds {
    fn bounding_box(&self) -> Rectangle;
}

impl<C, BO> GifFrameBounds for RawGifFrame<C, BO>
where
    C: PixelColor + From<C::Raw> + 'static,
    BO: ByteOrder + 'static,
{
    fn bounding_box(&self) -> Rectangle {
        self.bounding_box()
    }
}

impl<C> GifFrameBounds for RawGifCompressedFrame<C>
where
    C: PixelColor + 'static,
{
    fn bounding_box(&self) -> Rectangle {
        self.bounding_box()
    }
}

fn canvas_bounds<F>(frames: &[F]) -> Option<Rectangle>
where
    F: GifFrameBounds,
{
    let mut min_x = i32::MAX;
    let mut min_y = i32::MAX;
    let mut max_x = i32::MIN;
    let mut max_y = i32::MIN;

    for frame in frames {
        let frame_bounds = frame.bounding_box();
        min_x = min_x.min(frame_bounds.top_left.x);
        min_y = min_y.min(frame_bounds.top_left.y);
        max_x = max_x.max(frame_bounds.top_left.x + frame_bounds.size.width as i32);
        max_y = max_y.max(frame_bounds.top_left.y + frame_bounds.size.height as i32);
    }

    if min_x == i32::MAX {
        None
    } else {
        Some(Rectangle::new(
            Point::new(min_x, min_y),
            embedded_graphics::geometry::Size::new((max_x - min_x) as u32, (max_y - min_y) as u32),
        ))
    }
}

fn draw_compressed_frame<C, D>(
    target: &mut D,
    frame: &RawGifCompressedFrame<C>,
    origin: Point,
) -> Result<(), D::Error>
where
    C: CompressedColor + PixelColor + 'static,
    D: DrawTarget<Color = C>,
{
    let width = frame.size().width as usize;
    let height = frame.size().height as usize;
    let pixel_count = width.saturating_mul(height);
    let mut reader = CompressedPixels::<C>::new(frame.data(), pixel_count);

    target.draw_iter(core::iter::from_fn(move || {
        let (pixel_index, color) = reader.next_pixel()?;
        let x = pixel_index % width;
        let y = pixel_index / width;

        if y >= height {
            return None;
        }

        Some(Pixel(
            origin + frame.top_left() + Point::new(x as i32, y as i32),
            color,
        ))
    }))
}

const TOKEN_SKIP: u8 = 0b0000_0000;
const TOKEN_SOLID: u8 = 0b0100_0000;
const TOKEN_RAW: u8 = 0b1000_0000;
const TOKEN_KIND_MASK: u8 = 0b1100_0000;
const TOKEN_LEN_MASK: u8 = 0b0011_1111;

#[doc(hidden)]
pub trait CompressedColor: Copy {
    const BYTES_PER_PIXEL: usize;

    fn read(bytes: &[u8]) -> Option<Self>;
}

impl CompressedColor for Rgb888 {
    const BYTES_PER_PIXEL: usize = 3;

    fn read(bytes: &[u8]) -> Option<Self> {
        Some(Self::new(*bytes.first()?, *bytes.get(1)?, *bytes.get(2)?))
    }
}

impl CompressedColor for Rgb565 {
    const BYTES_PER_PIXEL: usize = 2;

    fn read(bytes: &[u8]) -> Option<Self> {
        let value = u16::from_be_bytes([*bytes.first()?, *bytes.get(1)?]);
        let red = ((value >> 11) & 0x1f) as u8;
        let green = ((value >> 5) & 0x3f) as u8;
        let blue = (value & 0x1f) as u8;
        Some(Self::new(red, green, blue))
    }
}

impl CompressedColor for BinaryColor {
    const BYTES_PER_PIXEL: usize = 1;

    fn read(bytes: &[u8]) -> Option<Self> {
        match *bytes.first()? {
            0 => Some(Self::Off),
            _ => Some(Self::On),
        }
    }
}

struct CompressedPixels<'a, C>
where
    C: CompressedColor,
{
    data: &'a [u8],
    offset: usize,
    cursor: usize,
    pixel_count: usize,
    raw_remaining: usize,
    solid_remaining: usize,
    solid_color: Option<C>,
}

impl<'a, C> CompressedPixels<'a, C>
where
    C: CompressedColor,
{
    fn new(data: &'a [u8], pixel_count: usize) -> Self {
        Self {
            data,
            offset: 0,
            cursor: 0,
            pixel_count,
            raw_remaining: 0,
            solid_remaining: 0,
            solid_color: None,
        }
    }

    fn next_pixel(&mut self) -> Option<(usize, C)> {
        loop {
            if self.cursor >= self.pixel_count {
                return None;
            }

            if self.raw_remaining > 0 {
                let color = self.read_color()?;
                let index = self.cursor;
                self.cursor += 1;
                self.raw_remaining -= 1;
                return Some((index, color));
            }

            if self.solid_remaining > 0 {
                let color = self.solid_color?;
                let index = self.cursor;
                self.cursor += 1;
                self.solid_remaining -= 1;
                return Some((index, color));
            }

            let token = *self.data.get(self.offset)?;
            self.offset += 1;
            let len = usize::from(token & TOKEN_LEN_MASK) + 1;

            match token & TOKEN_KIND_MASK {
                TOKEN_SKIP => {
                    self.cursor = self.cursor.saturating_add(len);
                }
                TOKEN_SOLID => {
                    self.solid_color = Some(self.read_color()?);
                    self.solid_remaining = len;
                }
                TOKEN_RAW => {
                    self.raw_remaining = len;
                }
                _ => return None,
            }
        }
    }

    fn read_color(&mut self) -> Option<C> {
        let end = self.offset.checked_add(C::BYTES_PER_PIXEL)?;
        let color = C::read(self.data.get(self.offset..end)?)?;
        self.offset = end;
        Some(color)
    }
}
