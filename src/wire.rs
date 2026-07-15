use std::str;

use crate::error::{Error, Result};
use crate::model::{
    enforce_limit, ClockConfidence, ClockConfidenceLevel, ClockTimestamp, Limits, MediaObject,
    ObjectKey, ObjectKind, PayloadHash, Stage, StageTimestamp,
};

/// Four-byte prefix for every canonical media-object envelope.
pub const WIRE_MAGIC: [u8; 4] = *b"MOBJ";
/// Wire-format version implemented by this crate.
pub const WIRE_VERSION: u16 = 1;

const HEADER_LEN: usize = 12;

/// Encode an object into its deterministic v1 binary envelope.
///
/// # Errors
///
/// Returns an error when the object violates a model, hash, ordering, or safety
/// invariant.
pub fn encode(object: &MediaObject) -> Result<Vec<u8>> {
    object.validate()?;
    let total_len = encoded_len_unchecked(object);
    let body_len = total_len - HEADER_LEN;
    let body_len_u32 = u32::try_from(body_len).map_err(|_| Error::LimitExceeded {
        field: "envelope_body",
        actual: body_len,
        maximum: u32::MAX as usize,
    })?;

    let mut output = Vec::with_capacity(total_len);
    output.extend_from_slice(&WIRE_MAGIC);
    write_u16(&mut output, WIRE_VERSION);
    write_u16(&mut output, 0);
    write_u32(&mut output, body_len_u32);

    output.push(object.kind() as u8);
    output.push(u8::from(object.is_keyframe()));
    write_u16(&mut output, 0);
    write_key(&mut output, object.key())?;
    write_u64(&mut output, object.configuration_epoch());
    write_optional_timestamp(&mut output, object.deadline())?;
    write_optional_timestamp(&mut output, object.capture_timestamp())?;

    write_u16(
        &mut output,
        checked_u16("stage_timestamps", object.stage_timestamps().len())?,
    );
    for stage in object.stage_timestamps() {
        output.push(stage.stage() as u8);
        write_timestamp(&mut output, stage.timestamp())?;
    }

    write_u16(
        &mut output,
        checked_u16("dependencies", object.dependencies().len())?,
    );
    for dependency in object.dependencies() {
        write_key(&mut output, dependency)?;
    }

    write_u16(
        &mut output,
        checked_u16("metadata_entries", object.metadata().len())?,
    );
    for (key, value) in object.metadata() {
        write_string(&mut output, key)?;
        write_u32(&mut output, checked_u32("metadata_value", value.len())?);
        output.extend_from_slice(value);
    }

    write_u32(&mut output, checked_u32("payload", object.payload().len())?);
    output.extend_from_slice(object.payload());

    debug_assert_eq!(output.len(), total_len);
    Ok(output)
}

/// Return the exact encoded size after validating the object.
///
/// # Errors
///
/// Returns an error when the object violates a model, hash, ordering, or safety
/// invariant.
pub fn encoded_len(object: &MediaObject) -> Result<usize> {
    object.validate()?;
    Ok(encoded_len_unchecked(object))
}

/// Decode with the crate's hard safety ceilings.
///
/// # Errors
///
/// Returns an error for malformed, noncanonical, oversized, unsupported, or
/// hash-invalid input.
pub fn decode(input: &[u8]) -> Result<MediaObject> {
    decode_with_limits(input, Limits::HARD)
}

/// Decode with service-specific limits bounded by [`Limits::HARD`].
///
/// Counts and lengths are checked before allocating owned strings, vectors, or
/// payload bytes. The caller should also apply `max_envelope_bytes` to its
/// transport frame before buffering the complete input slice.
///
/// # Errors
///
/// Returns an error for invalid limits or malformed, noncanonical, oversized,
/// unsupported, or hash-invalid input.
pub fn decode_with_limits(input: &[u8], limits: Limits) -> Result<MediaObject> {
    limits.validate()?;
    enforce_limit("envelope", input.len(), limits.max_envelope_bytes)?;
    if input.len() < HEADER_LEN {
        return Err(Error::Truncated);
    }

    if input[..4] != WIRE_MAGIC {
        return Err(Error::InvalidMagic);
    }
    let version = u16::from_be_bytes([input[4], input[5]]);
    if version != WIRE_VERSION {
        return Err(Error::UnsupportedVersion(version));
    }
    let reserved = u16::from_be_bytes([input[6], input[7]]);
    if reserved != 0 {
        return Err(Error::ReservedField("header"));
    }
    let declared = usize::try_from(u32::from_be_bytes([
        input[8], input[9], input[10], input[11],
    ]))
    .map_err(|_| Error::LimitExceeded {
        field: "declared_body",
        actual: usize::MAX,
        maximum: limits.max_envelope_bytes,
    })?;
    let actual = input.len() - HEADER_LEN;
    if declared != actual {
        return Err(Error::DeclaredLengthMismatch { declared, actual });
    }

    let mut reader = Reader::new(&input[HEADER_LEN..]);
    let kind = ObjectKind::from_wire(reader.read_u8()?)?;
    let keyframe = read_bool(&mut reader, "keyframe")?;
    if reader.read_u16()? != 0 {
        return Err(Error::ReservedField("object"));
    }
    let key = read_key(&mut reader, limits)?;
    let configuration_epoch = reader.read_u64()?;
    let deadline = read_optional_timestamp(&mut reader, limits, "deadline_present")?;
    let capture_timestamp =
        read_optional_timestamp(&mut reader, limits, "capture_timestamp_present")?;

    let stage_timestamps = read_stage_timestamps(&mut reader, limits)?;
    let dependencies = read_dependencies(&mut reader, limits)?;
    let metadata = read_metadata(&mut reader, limits)?;

    let payload_len = reader.read_u32_as_usize("payload")?;
    enforce_limit("payload", payload_len, limits.max_payload_bytes)?;
    let payload = reader.read_exact(payload_len)?.to_vec();
    if reader.remaining() != 0 {
        return Err(Error::TrailingBytes(reader.remaining()));
    }

    let mut builder = MediaObject::builder(key, kind, payload)
        .with_keyframe(keyframe)
        .with_configuration_epoch(configuration_epoch);
    if let Some(deadline) = deadline {
        builder = builder.with_deadline(deadline);
    }
    if let Some(capture_timestamp) = capture_timestamp {
        builder = builder.with_capture_timestamp(capture_timestamp);
    }
    for timestamp in stage_timestamps {
        builder = builder.with_stage_timestamp(timestamp);
    }
    for dependency in dependencies {
        builder = builder.with_dependency(dependency);
    }
    for (metadata_key, value) in metadata {
        builder = builder.with_metadata(metadata_key, value);
    }
    builder.build_with_limits(limits)
}

fn read_stage_timestamps(reader: &mut Reader<'_>, limits: Limits) -> Result<Vec<StageTimestamp>> {
    let count = usize::from(reader.read_u16()?);
    enforce_limit("stage_timestamps", count, limits.max_stage_timestamps)?;
    let mut timestamps = Vec::with_capacity(count);
    for _ in 0..count {
        let stage = Stage::from_wire(reader.read_u8()?)?;
        let entry = StageTimestamp::new(stage, read_timestamp(reader, limits)?);
        if timestamps.last().is_some_and(|previous| previous >= &entry) {
            return Err(Error::NonCanonicalOrder("stage_timestamps"));
        }
        timestamps.push(entry);
    }
    Ok(timestamps)
}

fn read_dependencies(reader: &mut Reader<'_>, limits: Limits) -> Result<Vec<ObjectKey>> {
    let count = usize::from(reader.read_u16()?);
    enforce_limit("dependencies", count, limits.max_dependencies)?;
    let mut dependencies = Vec::with_capacity(count);
    for _ in 0..count {
        let dependency = read_key(reader, limits)?;
        if dependencies
            .last()
            .is_some_and(|previous| previous >= &dependency)
        {
            return Err(Error::NonCanonicalOrder("dependencies"));
        }
        dependencies.push(dependency);
    }
    Ok(dependencies)
}

fn read_metadata(reader: &mut Reader<'_>, limits: Limits) -> Result<Vec<(String, Vec<u8>)>> {
    let count = usize::from(reader.read_u16()?);
    enforce_limit("metadata_entries", count, limits.max_metadata_entries)?;
    let mut metadata = Vec::with_capacity(count);
    let mut total_bytes = 0usize;
    for _ in 0..count {
        let key = reader.read_string(limits.max_metadata_key_bytes, "metadata_key")?;
        if metadata
            .last()
            .is_some_and(|(previous, _): &(String, Vec<u8>)| previous >= &key)
        {
            return Err(Error::NonCanonicalOrder("metadata"));
        }
        let value_len = reader.read_u32_as_usize("metadata_value")?;
        enforce_limit("metadata_value", value_len, limits.max_metadata_value_bytes)?;
        total_bytes = total_bytes
            .checked_add(key.len())
            .and_then(|sum| sum.checked_add(value_len))
            .ok_or(Error::LimitExceeded {
                field: "metadata",
                actual: usize::MAX,
                maximum: limits.max_metadata_bytes,
            })?;
        enforce_limit("metadata", total_bytes, limits.max_metadata_bytes)?;
        metadata.push((key, reader.read_exact(value_len)?.to_vec()));
    }
    Ok(metadata)
}

pub(crate) fn encoded_len_unchecked(object: &MediaObject) -> usize {
    let mut length = HEADER_LEN + 1 + 1 + 2;
    length += key_len(object.key());
    length += 8;
    length += optional_timestamp_len(object.deadline());
    length += optional_timestamp_len(object.capture_timestamp());
    length += 2;
    for stage in object.stage_timestamps() {
        length += 1 + timestamp_len(stage.timestamp());
    }
    length += 2;
    for dependency in object.dependencies() {
        length += key_len(dependency);
    }
    length += 2;
    for (key, value) in object.metadata() {
        length += string_len(key) + 4 + value.len();
    }
    length + 4 + object.payload().len()
}

fn write_key(output: &mut Vec<u8>, key: &ObjectKey) -> Result<()> {
    write_string(output, key.tenant())?;
    write_string(output, key.stream())?;
    write_string(output, key.track())?;
    write_u64(output, key.epoch());
    write_u64(output, key.group());
    write_u64(output, key.object());
    write_u32(output, key.version());
    output.extend_from_slice(key.payload_hash().as_bytes());
    Ok(())
}

fn read_key(reader: &mut Reader<'_>, limits: Limits) -> Result<ObjectKey> {
    let tenant = reader.read_string(limits.max_identifier_bytes, "tenant")?;
    let stream = reader.read_string(limits.max_identifier_bytes, "stream")?;
    let track = reader.read_string(limits.max_identifier_bytes, "track")?;
    let epoch = reader.read_u64()?;
    let group = reader.read_u64()?;
    let object = reader.read_u64()?;
    let version = reader.read_u32()?;
    let payload_hash = PayloadHash::from_bytes(reader.read_array::<32>()?);
    ObjectKey::new(
        tenant,
        stream,
        track,
        epoch,
        group,
        object,
        version,
        payload_hash,
    )
}

fn write_optional_timestamp(
    output: &mut Vec<u8>,
    timestamp: Option<&ClockTimestamp>,
) -> Result<()> {
    if let Some(timestamp) = timestamp {
        output.push(1);
        write_timestamp(output, timestamp)?;
    } else {
        output.push(0);
    }
    Ok(())
}

fn read_optional_timestamp(
    reader: &mut Reader<'_>,
    limits: Limits,
    field: &'static str,
) -> Result<Option<ClockTimestamp>> {
    if read_bool(reader, field)? {
        Ok(Some(read_timestamp(reader, limits)?))
    } else {
        Ok(None)
    }
}

fn write_timestamp(output: &mut Vec<u8>, timestamp: &ClockTimestamp) -> Result<()> {
    write_i64(output, timestamp.unix_time_ns());
    write_string(output, timestamp.clock_id())?;
    output.push(timestamp.confidence().level() as u8);
    if let Some(maximum_error_ns) = timestamp.confidence().maximum_error_ns() {
        output.push(1);
        write_u64(output, maximum_error_ns);
    } else {
        output.push(0);
    }
    Ok(())
}

fn read_timestamp(reader: &mut Reader<'_>, limits: Limits) -> Result<ClockTimestamp> {
    let unix_time_ns = reader.read_i64()?;
    let clock_id = reader.read_string(limits.max_clock_id_bytes, "clock_id")?;
    let confidence_level = ClockConfidenceLevel::from_wire(reader.read_u8()?)?;
    let maximum_error_ns = if read_bool(reader, "clock_error_present")? {
        Some(reader.read_u64()?)
    } else {
        None
    };
    let confidence = ClockConfidence::from_parts(confidence_level, maximum_error_ns)?;
    ClockTimestamp::new(unix_time_ns, clock_id, confidence)
}

fn read_bool(reader: &mut Reader<'_>, field: &'static str) -> Result<bool> {
    match reader.read_u8()? {
        0 => Ok(false),
        1 => Ok(true),
        value => Err(Error::InvalidBoolean { field, value }),
    }
}

fn key_len(key: &ObjectKey) -> usize {
    string_len(key.tenant()) + string_len(key.stream()) + string_len(key.track()) + 24 + 4 + 32
}

fn optional_timestamp_len(timestamp: Option<&ClockTimestamp>) -> usize {
    1 + timestamp.map_or(0, timestamp_len)
}

fn timestamp_len(timestamp: &ClockTimestamp) -> usize {
    8 + string_len(timestamp.clock_id())
        + 1
        + 1
        + timestamp.confidence().maximum_error_ns().map_or(0, |_| 8)
}

fn string_len(value: &str) -> usize {
    2 + value.len()
}

fn write_string(output: &mut Vec<u8>, value: &str) -> Result<()> {
    write_u16(output, checked_u16("string", value.len())?);
    output.extend_from_slice(value.as_bytes());
    Ok(())
}

fn checked_u16(field: &'static str, value: usize) -> Result<u16> {
    u16::try_from(value).map_err(|_| Error::LimitExceeded {
        field,
        actual: value,
        maximum: usize::from(u16::MAX),
    })
}

fn checked_u32(field: &'static str, value: usize) -> Result<u32> {
    u32::try_from(value).map_err(|_| Error::LimitExceeded {
        field,
        actual: value,
        maximum: u32::MAX as usize,
    })
}

fn write_u16(output: &mut Vec<u8>, value: u16) {
    output.extend_from_slice(&value.to_be_bytes());
}

fn write_u32(output: &mut Vec<u8>, value: u32) {
    output.extend_from_slice(&value.to_be_bytes());
}

fn write_u64(output: &mut Vec<u8>, value: u64) {
    output.extend_from_slice(&value.to_be_bytes());
}

fn write_i64(output: &mut Vec<u8>, value: i64) {
    output.extend_from_slice(&value.to_be_bytes());
}

struct Reader<'a> {
    input: &'a [u8],
    offset: usize,
}

impl<'a> Reader<'a> {
    const fn new(input: &'a [u8]) -> Self {
        Self { input, offset: 0 }
    }

    fn read_exact(&mut self, length: usize) -> Result<&'a [u8]> {
        let end = self.offset.checked_add(length).ok_or(Error::Truncated)?;
        let value = self.input.get(self.offset..end).ok_or(Error::Truncated)?;
        self.offset = end;
        Ok(value)
    }

    fn read_array<const N: usize>(&mut self) -> Result<[u8; N]> {
        self.read_exact(N)?.try_into().map_err(|_| Error::Truncated)
    }

    fn read_u8(&mut self) -> Result<u8> {
        Ok(self.read_exact(1)?[0])
    }

    fn read_u16(&mut self) -> Result<u16> {
        Ok(u16::from_be_bytes(self.read_array()?))
    }

    fn read_u32(&mut self) -> Result<u32> {
        Ok(u32::from_be_bytes(self.read_array()?))
    }

    fn read_u32_as_usize(&mut self, field: &'static str) -> Result<usize> {
        usize::try_from(self.read_u32()?).map_err(|_| Error::LimitExceeded {
            field,
            actual: usize::MAX,
            maximum: usize::MAX,
        })
    }

    fn read_u64(&mut self) -> Result<u64> {
        Ok(u64::from_be_bytes(self.read_array()?))
    }

    fn read_i64(&mut self) -> Result<i64> {
        Ok(i64::from_be_bytes(self.read_array()?))
    }

    fn read_string(&mut self, maximum: usize, field: &'static str) -> Result<String> {
        let length = usize::from(self.read_u16()?);
        enforce_limit(field, length, maximum)?;
        let bytes = self.read_exact(length)?;
        let value = str::from_utf8(bytes).map_err(|_| Error::InvalidUtf8(field))?;
        Ok(value.to_owned())
    }

    fn remaining(&self) -> usize {
        self.input.len() - self.offset
    }
}
