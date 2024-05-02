use std::fs::File;

use crate::{
    pipeline::{
        GGMLLoaderBuilder, GGMLSpecificConfig, GGUFLoaderBuilder, GGUFSpecificConfig,
        NormalSpecificConfig,
    },
    Loader, ModelSelected, NormalLoaderBuilder, SpeculativeConfig, SpeculativeLoader,
};

pub struct LoaderBuilder {
    model: ModelSelected,
    no_kv_cache: bool,
    chat_template: Option<String>,
    use_flash_attn: bool,
}

impl LoaderBuilder {
    pub fn new(model: ModelSelected) -> Self {
        Self {
            model,
            no_kv_cache: false,
            chat_template: None,
            use_flash_attn: false,
        }
    }

    pub fn with_no_kv_cache(mut self, no_kv_cache: bool) -> Self {
        self.no_kv_cache = no_kv_cache;
        self
    }
    pub fn with_chat_template(mut self, chat_template: Option<String>) -> Self {
        self.chat_template = chat_template;
        self
    }
    pub fn with_use_flash_attn(mut self, use_flash_attn: bool) -> Self {
        self.use_flash_attn = use_flash_attn;
        self
    }

    pub fn build(self) -> anyhow::Result<Box<dyn Loader>> {
        loader_from_model_selected(self)
    }
}

pub fn get_tgt_non_granular_index(model: &ModelSelected) -> Option<usize> {
    match model {
        ModelSelected::Plain { .. }
        | ModelSelected::Lora { .. }
        | ModelSelected::GGUF { .. }
        | ModelSelected::LoraGGUF { .. }
        | ModelSelected::GGML { .. }
        | ModelSelected::LoraGGML { .. }
        | ModelSelected::SpeculativeGGUF { .. } => None,
        ModelSelected::XLora {
            tgt_non_granular_index,
            ..
        }
        | ModelSelected::XLoraGGUF {
            tgt_non_granular_index,
            ..
        }
        | ModelSelected::XLoraGGML {
            tgt_non_granular_index,
            ..
        } => *tgt_non_granular_index,
    }
}

fn loader_from_model_selected(args: LoaderBuilder) -> anyhow::Result<Box<dyn Loader>> {
    let use_flash_attn = args.use_flash_attn;
    let tgt_non_granular_index = get_tgt_non_granular_index(&args.model);
    let loader: Box<dyn Loader> = match args.model {
        ModelSelected::Plain {
            model_id,
            repeat_last_n,
            tokenizer_json,
            arch,
        } => NormalLoaderBuilder::new(
            NormalSpecificConfig {
                use_flash_attn,
                repeat_last_n,
            },
            args.chat_template,
            tokenizer_json,
            Some(model_id),
        )
        .build(arch),
        ModelSelected::XLora {
            model_id,
            xlora_model_id,
            repeat_last_n,
            order,
            tokenizer_json,
            tgt_non_granular_index,
            arch,
        } => NormalLoaderBuilder::new(
            NormalSpecificConfig {
                use_flash_attn,
                repeat_last_n,
            },
            args.chat_template,
            tokenizer_json,
            model_id,
        )
        .with_xlora(
            xlora_model_id,
            serde_json::from_reader(
                File::open(order.clone())
                    .unwrap_or_else(|_| panic!("Could not load ordering file at {order}")),
            )?,
            args.no_kv_cache,
            tgt_non_granular_index,
        )
        .build(arch),
        ModelSelected::Lora {
            model_id,
            tokenizer_json,
            adapters_model_id,
            repeat_last_n,
            order,
            arch,
        } => NormalLoaderBuilder::new(
            NormalSpecificConfig {
                use_flash_attn,
                repeat_last_n,
            },
            args.chat_template,
            tokenizer_json,
            model_id,
        )
        .with_lora(
            adapters_model_id,
            serde_json::from_reader(
                File::open(order.clone())
                    .unwrap_or_else(|_| panic!("Could not load ordering file at {order}")),
            )?,
            args.no_kv_cache,
            tgt_non_granular_index,
        )
        .build(arch),
        ModelSelected::GGUF {
            tok_model_id,
            tokenizer_json,
            quantized_model_id,
            quantized_filename,
            repeat_last_n,
        } => GGUFLoaderBuilder::new(
            GGUFSpecificConfig { repeat_last_n },
            args.chat_template,
            tokenizer_json,
            Some(tok_model_id),
            quantized_model_id,
            quantized_filename,
        )
        .build(),
        ModelSelected::XLoraGGUF {
            tok_model_id,
            tokenizer_json,
            quantized_model_id,
            quantized_filename,
            repeat_last_n,
            xlora_model_id,
            order,
            tgt_non_granular_index,
        } => GGUFLoaderBuilder::new(
            GGUFSpecificConfig { repeat_last_n },
            args.chat_template,
            tokenizer_json,
            tok_model_id,
            quantized_model_id,
            quantized_filename,
        )
        .with_xlora(
            xlora_model_id,
            serde_json::from_reader(
                File::open(order.clone())
                    .unwrap_or_else(|_| panic!("Could not load ordering file at {order}")),
            )?,
            args.no_kv_cache,
            tgt_non_granular_index,
        )
        .build(),
        ModelSelected::LoraGGUF {
            tok_model_id,
            tokenizer_json,
            quantized_model_id,
            quantized_filename,
            repeat_last_n,
            adapters_model_id,
            order,
            tgt_non_granular_index,
        } => GGUFLoaderBuilder::new(
            GGUFSpecificConfig { repeat_last_n },
            args.chat_template,
            tokenizer_json,
            tok_model_id,
            quantized_model_id,
            quantized_filename,
        )
        .with_lora(
            adapters_model_id,
            serde_json::from_reader(
                File::open(order.clone())
                    .unwrap_or_else(|_| panic!("Could not load ordering file at {order}")),
            )?,
            args.no_kv_cache,
            tgt_non_granular_index,
        )
        .build(),
        ModelSelected::GGML {
            tok_model_id,
            tokenizer_json,
            quantized_model_id,
            quantized_filename,
            repeat_last_n,
            gqa,
        } => GGMLLoaderBuilder::new(
            GGMLSpecificConfig { repeat_last_n, gqa },
            args.chat_template,
            tokenizer_json,
            Some(tok_model_id),
            quantized_model_id,
            quantized_filename,
        )
        .build(),
        ModelSelected::XLoraGGML {
            tok_model_id,
            tokenizer_json,
            quantized_model_id,
            quantized_filename,
            repeat_last_n,
            xlora_model_id,
            order,
            tgt_non_granular_index,
            gqa,
        } => GGMLLoaderBuilder::new(
            GGMLSpecificConfig { repeat_last_n, gqa },
            args.chat_template,
            tokenizer_json,
            tok_model_id,
            quantized_model_id,
            quantized_filename,
        )
        .with_xlora(
            xlora_model_id,
            serde_json::from_reader(
                File::open(order.clone())
                    .unwrap_or_else(|_| panic!("Could not load ordering file at {order}")),
            )?,
            args.no_kv_cache,
            tgt_non_granular_index,
        )
        .build(),
        ModelSelected::LoraGGML {
            tok_model_id,
            tokenizer_json,
            quantized_model_id,
            quantized_filename,
            repeat_last_n,
            adapters_model_id,
            order,
            tgt_non_granular_index,
            gqa,
        } => GGMLLoaderBuilder::new(
            GGMLSpecificConfig { repeat_last_n, gqa },
            args.chat_template,
            tokenizer_json,
            tok_model_id,
            quantized_model_id,
            quantized_filename,
        )
        .with_lora(
            adapters_model_id,
            serde_json::from_reader(
                File::open(order.clone())
                    .unwrap_or_else(|_| panic!("Could not load ordering file at {order}")),
            )?,
            args.no_kv_cache,
            tgt_non_granular_index,
        )
        .build(),
        ModelSelected::SpeculativeGGUF {
            tok_model_id,
            tokenizer_json,
            quantized_model_id,
            quantized_filename,
            draft_quantized_model_id,
            draft_quantized_filename,
            repeat_last_n,
        } => {
            let target_loader = GGUFLoaderBuilder::new(
                GGUFSpecificConfig { repeat_last_n },
                args.chat_template.clone(),
                tokenizer_json.clone(),
                Some(tok_model_id.clone()),
                quantized_model_id,
                quantized_filename,
            )
            .build();
            let draft_loader = GGUFLoaderBuilder::new(
                GGUFSpecificConfig { repeat_last_n },
                args.chat_template,
                tokenizer_json,
                Some(tok_model_id),
                draft_quantized_model_id,
                draft_quantized_filename,
            )
            .build();
            Box::new(SpeculativeLoader {
                target: target_loader,
                draft: draft_loader,
                config: SpeculativeConfig { gamma: 5 },
            })
        }
    };
    Ok(loader)
}
