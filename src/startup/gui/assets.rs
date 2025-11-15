pub mod font {
    pub const JETBRAINS_MONO_BOLD_BYTES: &[u8] = include_bytes!("../../../assets/font/JetBrainsMono-Bold.ttf");

    #[cfg(feature = "steam")]
    pub const JETBRAINS_MONO_NL_EXTRA_LIGHT_BYTES: &[u8] = include_bytes!("../../../assets/font/JetBrainsMonoNL-ExtraLight.ttf");
}
