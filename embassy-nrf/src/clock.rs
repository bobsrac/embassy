//! Clock

pub use hfclk::{hfclk_source, set_hfclk_source};
use crate::chip::pac::clock::vals::HfclkstatSrc;

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
/// High frequency clock source.
pub enum HfclkSource {
    /// Internal source
    #[default]
    Internal,
    /// External source from xtal.
    ExternalXtal,
}
impl From<HfclkstatSrc> for HfclkSource {
    fn from(src: HfclkstatSrc) -> Self {
        match src {
            HfclkstatSrc::RC => Self::Internal,
            HfclkstatSrc::XTAL => Self::ExternalXtal,
        }
    }
}
impl From<HfclkSource> for HfclkstatSrc {
    fn from(src: HfclkSource) -> Self {
        match src {
            HfclkSource::Internal => Self::RC,
            HfclkSource::ExternalXtal => Self::XTAL,
        }
    }
}

// NOTE: _nrf54l code untested
#[cfg(feature = "_nrf54l")]
pub use hfclk::{Pll, PllFreq, pll, pll_freq, set_pll_freq};
#[cfg(feature = "_nrf54l")]
mod hfclk {
    use crate::chip::pac::CLOCK_NS;
    use crate::chip::pac::OSCILLATORS_NS;
    use crate::chip::pac::oscillators::vals::{CurrentFreq, Freq};
    use super::HfclkSource;

    pub fn hfclk_source() -> HfclkSource {
        CLOCK_NS::xo().stat().state().map(|running| match running {
            false => HfclkSource::Internal,
            true => hfclkSource::ExternalXtal,
        })
    }

    pub fn set_hfclk_source(source: HfclkSource) {
        if hfclk_source() != freq {
            match source {
                HfclkSource::ExternalXtal => {
                    CLOCK_NS.events_xostarted().write_value(0);
                    CLOCK_NS.tasks_xostart().write_value(1);
                    while CLOCK_NS.events_xostarted().read() == 0 {}
                },
                HfclkSource::Internal => { CLOCK_NS.tasks_xostop().write_value(1); },
            }
        }
    }

    pub struct Pll {
        /// Requested speed of MCU power domain, including CPU
        freq: PllFreq,
        /// Current speed of MCU power domain, including CPU
        current_freq: PllFreq,
        /// PLL is running
        running: bool,
    }
    pub enum PllFreq {
        /// 128 MHz
        Clk128Mhz,
        /// 64 MHz
        Clk64Mhz,
    }
    impl From<CurrentFreq> for PllFreq {
        fn from(freq: CurrentFreq) -> Self {
            match freq {
                Currentfreq::CK64M => Self::Clk64Mhz,
                Currentfreq::CK128M => Self::Clk128MHz,
            }
        }
    }
    impl From<Freq> for PllFreq {
        fn from(freq: Freq) -> Self {
            match freq {
                Freq::CK64M => Self::Clk64Mhz,
                Freq::CK128M => Self::Clk128MHz,
            }
        }
    }
    impl From<PllFreq> for Currentfreq {
        fn from(freq: PllFreq) -> Self {
            match freq {
                PllFreq::Clk64MHz => Self::Ck64M,
                PllFreq::Clk128MHz => Self::Ck128M,
            }
        }
    }
    impl From<PllFreq> for Freq {
        fn from(freq: Freq) -> Self {
            match freq {
                PllFreq::Clk64MHz => Self::Ck64M,
                PllFreq::Clk128MHz => Self::Ck128M,
            }
        }
    }

    pub fn pll() -> Pll {
        Pll {
            freq: OSCILLATORS_NS::pll().freq().freq().into(),
            current_freq: OSCILLATORS_NS::pll().currentfreq().currentfreq().into(),
            running: CLOCK_NS::pll().stat().state()
        }
    }

    pub fn pll_freq() -> PllFreq {
        OSCILLATORS_NS::pll().freq().freq().into()
    }

    pub fn set_pll_freq(freq: PllFreq) {
        if pll_freq() != freq {
            OSCILLATORS_NS::pll().freq().set_freq(freq.into());
        }
    }
}

#[cfg(not(feature = "_nrf54l"))]
pub use hfclk::{hfxo_debounce_micros, try_set_hfxo_debounce_micros};
#[cfg(not(feature = "_nrf54l"))]
mod hfclk {
    use crate::chip::pac::CLOCK;
    use crate::chip::pac::clock::vals::Hfxodebounce;
    use super::HfclkSource;

    /// HFCLK Source
    pub fn hfclk_source() -> HfclkSource {
        CLOCK.hfclkstat().read().src().into()
    }

    /// Set HFCLK Source
    pub fn set_hfclk_source(source: HfclkSource) {
        if hfclk_source() != source {
            match source {
                HfclkSource::ExternalXtal => {
                    CLOCK.events_hfclkstarted().write_value(0);
                    CLOCK.tasks_hfclkstart().write_value(1);
                    while CLOCK.events_hfclkstarted().read() == 0 {}
                },
                HfclkSource::Internal => { CLOCK.tasks_hfclkstop().write_value(1); },
            }
        }
    }

    /// HFXO Debounce Time in microseconds
    pub fn hfxo_debounce_micros() -> u16 {
        (CLOCK.hfxodebounce().read().hfxodebounce().0 as u16) * 16
    }

    /// Set HFXO Debounce Time in microseconds.
    ///   Rounded to a multiple of 16; Min 8, Max 4087
    pub fn try_set_hfxo_debounce_micros(micros: u16) -> Result<(), Error> {
        let target_reg_counts = (micros as u32 + 8) / 16;
        if target_reg_counts < 1 {
            return Err(Error::HfxoDebounceTooSmall);
        }
        if target_reg_counts > 0xff {
            return Err(Error::HfxoDebounceTooLarge);
        }
        if hfxo_debounce_micros() != target_reg_counts as u16 * 16 {
            CLOCK.hfxodebounce().modify(|h| h.set_hfxodebounce(Hfxodebounce(target_reg_counts as u8)));
        }
        Ok(())
    }

    #[derive(Debug, Copy, Clone, Eq, PartialEq)]
    #[cfg_attr(feature = "defmt", derive(defmt::Format))]
    #[non_exhaustive]
    pub enum Error {
        /// Requested hfxo_debounce value below minimum
        HfxoDebounceTooSmall,
        /// Requested hfxo_debounce value above maximum
        HfxoDebounceTooLarge,
    }
}
