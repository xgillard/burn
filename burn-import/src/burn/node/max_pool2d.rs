use proc_macro2::TokenStream;
use quote::quote;

use burn::{nn::{pool::MaxPool2dConfig, PaddingConfig2d}, record::PrecisionSettings, tensor::backend::Backend, module::{Module, ConstantRecord}};
use serde::Serialize;

use super::{Node, NodeCodegen, SerializationBackend};
use crate::burn::{BurnImports, OtherType, Scope, TensorType, ToTokens, Type};

#[derive(Debug, Clone)]
pub struct MaxPool2dNode {
    pub field: OtherType,
    pub input: TensorType,
    pub output: TensorType,
    pub config: MaxPool2dConfig,
}

impl MaxPool2dNode {
    pub fn new<S: AsRef<str>>(
        name: S,
        input: TensorType,
        output: TensorType,
        config: MaxPool2dConfig,
    ) -> Self {
        Self {
            field: OtherType::new(
                name,
                quote! {
                    MaxPool2d<B>
                },
            ),
            input,
            output,
            config,
        }
    }
}

/// I guess this should be auto generated but I have no idea of **HOW** it 
/// should be done.
#[derive(burn::record::Record)]
pub struct MaxPool2dRecord<B:Backend> {
    pub channels: <usize as Module<B>>::Record,
    pub kernel_size: <[usize; 2] as Module<B>>::Record,
    pub strides: <[usize; 2] as Module<B>>::Record,
    pub padding: <PaddingConfig2d as Module<B>>::Record,
}

impl<PS: PrecisionSettings> NodeCodegen<PS> for MaxPool2dNode {
    fn input_types(&self) -> Vec<Type> {
        vec![Type::Tensor(&self.input)]
    }
    fn output_types(&self) -> Vec<Type> {
        vec![Type::Tensor(&self.output)]
    }
    fn field_type(&self) -> Option<Type> {
        Some(Type::Other(&self.field))
    }

    fn field_init(&self, _with_record: bool) -> Option<TokenStream> {
        let name = &self.field.name;
        let channels = self.config.channels.to_tokens();
        let kernel_size = self.config.kernel_size.to_tokens();
        let strides = self.config.strides.to_tokens();
        let padding = self.config.padding.to_tokens();

        let init_line = quote! {
            init();
        };

        let tokens = quote! {
            let #name = MaxPool2dConfig::new(#channels, #kernel_size)
                .with_strides(#strides)
                .with_padding(#padding)
                .#init_line
        };

        Some(tokens)
    }

    fn field_serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let record = MaxPool2dRecord::<SerializationBackend> {
            channels: ConstantRecord::new(),
            kernel_size: [ConstantRecord::new(); 2],
            strides: [ConstantRecord::new(); 2],
            padding: ConstantRecord::new(),
        };

        let item = burn::record::Record::into_item::<PS>(record);
        item.serialize(serializer)
    }

    fn forward(&self, scope: &mut Scope, node_position: usize) -> TokenStream {
        let input = scope.tensor_use_owned(&self.input, node_position);
        let output = &self.output.name;
        let field = &self.field.name;

        quote! {
            let #output = self.#field.forward(#input);
        }
    }
    fn register_imports(&self, imports: &mut BurnImports) {
        imports.register("burn::nn::PaddingConfig2d");
        imports.register("burn::nn::pool::MaxPool2d");
        imports.register("burn::nn::pool::MaxPool2dConfig");
    }

    fn into_node(self) -> Node<PS> {
        Node::MaxPool2d(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::burn::{
        graph::BurnGraph,
        node::{max_pool2d::MaxPool2dNode, test::assert_tokens},
        TensorType,
    };
    use burn::{nn::pool::MaxPool2dConfig, nn::PaddingConfig2d, record::FullPrecisionSettings};

    #[test]
    fn test_codegen() {
        let mut graph = BurnGraph::<FullPrecisionSettings>::default();

        graph.register(MaxPool2dNode::new(
            "max_pool2d",
            TensorType::new_float("input", 4),
            TensorType::new_float("output", 4),
            MaxPool2dConfig::new(1, [3, 3])
                .with_strides([1, 1])
                .with_padding(PaddingConfig2d::Valid),
        ));

        let expected = quote! {
            use burn::{
                module::Module,
                tensor::{backend::Backend, Tensor},
            };
            use burn::nn::PaddingConfig2d;
            use burn::nn::pool::MaxPool2d;
            use burn::nn::pool::MaxPool2dConfig;

            #[derive(Module, Debug)]
            pub struct Model <B: Backend> {
                max_pool2d: MaxPool2d<B>,
            }

            impl<B: Backend> Model <B> {
                pub fn new_with(record: ModelRecord<B>) -> Self {
                    let max_pool2d = MaxPool2dConfig::new(1, [3, 3])
                        .with_strides([1, 1])
                        .with_padding(PaddingConfig2d::Valid)
                        .init();

                    Self {
                        max_pool2d,
                    }
                }
                #[allow(clippy::let_and_return)]
                pub fn forward(&self, input: Tensor<B, 4>) -> Tensor<B, 4> {
                    let output = self.max_pool2d.forward(input);

                    output
                }
            }
        };

        assert_tokens(graph.codegen(), expected);
    }
}
