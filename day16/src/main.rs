use anyhow::anyhow;
use std::fs;
use std::str::Chars;

#[derive(Debug)]
struct BITSPacket {
    version: u8,
    payload: Payload,
}

impl BITSPacket {
    fn parse(iter: &mut BitsIter) -> anyhow::Result<Self> {
        let version = Self::read_to_u8(iter, 3);
        let type_id = Self::read_to_u8(iter, 3);

        match type_id {
            4 => {
                let literal_value = Self::read_literal(iter);
                Ok(Self {
                    version,
                    payload: Payload::LiteralValue(literal_value),
                })
            }
            operator_id => {
                let length_type_id = Self::read_to_u8(iter, 1);

                match length_type_id {
                    0 => {
                        let total_bits_len = Self::read_to_u16(iter, 15) as usize;
                        let mut processed_bits_len = 0;
                        let mut subpackets = vec![];

                        while processed_bits_len != total_bits_len {
                            let processed_before = iter.processed_bits();
                            subpackets.push(Self::parse(iter)?);
                            processed_bits_len += iter.processed_bits() - processed_before;
                        }

                        Ok(Self {
                            version,
                            payload: Payload::OperatorPayload(
                                operator_type(operator_id)?,
                                subpackets,
                            ),
                        })
                    }
                    1 => {
                        let subpackets_count = Self::read_to_u16(iter, 11) as usize;

                        Ok(Self {
                            version,
                            payload: Payload::OperatorPayload(
                                operator_type(operator_id)?,
                                (0..subpackets_count)
                                    .map(|_| Self::parse(iter))
                                    .collect::<Result<_, _>>()?,
                            ),
                        })
                    }
                    _ => return Err(anyhow!("Unknown length type ID - {}", length_type_id)),
                }
            }
        }
    }

    fn read_to_u8(iter: &mut BitsIter, len: usize) -> u8 {
        let mut result = 0;

        for (i, bit) in iter.take(len).enumerate() {
            if bit == 1 {
                result |= 1 << (len - i - 1);
            }
        }

        result
    }

    fn read_to_u16(iter: &mut BitsIter, len: usize) -> u16 {
        let mut result = 0;

        for (i, bit) in iter.take(len).enumerate() {
            if bit == 1 {
                result |= 1 << (len - i - 1);
            }
        }

        result
    }

    fn read_literal(iter: &mut BitsIter) -> usize {
        let mut bytes = Vec::with_capacity(8);

        loop {
            let piece = Self::read_to_u16(iter, 5);
            let value = piece & 0x0F;
            let continue_bit = piece >> 4;

            bytes.push(value);
            if continue_bit == 0 {
                break;
            }
        }

        let total_nibbles = bytes.len();

        bytes
            .into_iter()
            .enumerate()
            .fold(0usize, |total, (i, byte)| {
                total | ((byte as usize) << ((total_nibbles - 1 - i) * 4))
            })
    }

    fn evaluate(&self) -> anyhow::Result<usize> {
        use OperatorType::*;
        match &self.payload {
            Payload::LiteralValue(value) => Ok(*value),
            Payload::OperatorPayload(operator, data) => {
                let evaluated_payload: Vec<usize> = data
                    .iter()
                    .map(BITSPacket::evaluate)
                    .collect::<Result<_, _>>()?;

                match *operator {
                    Sum => Ok(evaluated_payload.into_iter().sum()),
                    Product => Ok(evaluated_payload.into_iter().product()),
                    Maximum => evaluated_payload
                        .into_iter()
                        .max()
                        .ok_or(anyhow!("failed to find maximum in payload")),
                    Minimum => evaluated_payload
                        .into_iter()
                        .min()
                        .ok_or(anyhow!("failed to find minimum in payload")),
                    GreaterThan => Ok(if evaluated_payload[0] > evaluated_payload[1] {
                        1
                    } else {
                        0
                    }),
                    LessThan => Ok(if evaluated_payload[0] < evaluated_payload[1] {
                        1
                    } else {
                        0
                    }),
                    EqualTo => Ok(if evaluated_payload[0] == evaluated_payload[1] {
                        1
                    } else {
                        0
                    }),
                }
            }
        }
    }
}

impl<'iter> TryFrom<BitsIter<'iter>> for BITSPacket {
    type Error = anyhow::Error;

    fn try_from(mut iter: BitsIter<'iter>) -> Result<Self, Self::Error> {
        BITSPacket::parse(&mut iter)
    }
}

#[derive(Debug, Clone, Copy)]
enum OperatorType {
    Sum,
    Product,
    Maximum,
    Minimum,
    GreaterThan,
    LessThan,
    EqualTo,
}

fn operator_type(operation_id: u8) -> anyhow::Result<OperatorType> {
    use OperatorType::*;

    match operation_id {
        0 => Ok(Sum),
        1 => Ok(Product),
        2 => Ok(Minimum),
        3 => Ok(Maximum),
        5 => Ok(GreaterThan),
        6 => Ok(LessThan),
        7 => Ok(EqualTo),
        _ => Err(anyhow!("unknown operation id: {}", operation_id)),
    }
}

#[derive(Debug)]
enum Payload {
    LiteralValue(usize),
    OperatorPayload(OperatorType, Vec<BITSPacket>),
}

#[derive(Clone)]
struct BitsIter<'packet> {
    chars: Chars<'packet>,
    bit_idx: u8,
    current: u8,
    processed_bits: usize,
}

impl<'packet> BitsIter<'packet> {
    fn take(s: &'packet str) -> Self {
        Self {
            chars: s.chars(),
            bit_idx: 4,
            current: 0,
            processed_bits: 0,
        }
    }

    fn as_nibble(hex: char) -> u8 {
        if ('0'..='9').contains(&hex) {
            hex as u8 - '0' as u8
        } else {
            hex as u8 - 'A' as u8 + 10
        }
    }

    fn processed_bits(&self) -> usize {
        self.processed_bits
    }
}

impl<'packet> Iterator for BitsIter<'packet> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        if self.bit_idx == 4 {
            self.current = Self::as_nibble(self.chars.next()?);
            self.bit_idx = 0;
        }

        let bit = self.current >> (3 - self.bit_idx) & 1;
        self.bit_idx += 1;
        self.processed_bits += 1;
        Some(bit)
    }
}

fn version_sum(packet: &BITSPacket) -> usize {
    let main_version = packet.version as usize;
    match &packet.payload {
        Payload::LiteralValue(_) => main_version,
        Payload::OperatorPayload(_, subpackets) => {
            main_version + subpackets.iter().map(version_sum).sum::<usize>()
        }
    }
}

fn main() -> anyhow::Result<()> {
    let bits_packet = fs::read_to_string("./input")?;
    let packet = BITSPacket::try_from(BitsIter::take(&bits_packet))?;

    println!("Version sum of sent BITS packet: {}", version_sum(&packet));
    println!(
        "Evaluated result of sent BITS packet: {}",
        packet.evaluate()?
    );
    Ok(())
}
