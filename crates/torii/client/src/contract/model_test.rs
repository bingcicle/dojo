use camino::Utf8PathBuf;
use dojo_test_utils::sequencer::{
    get_default_test_starknet_config, SequencerConfig, TestSequencer,
};
use dojo_types::core::CairoType;
use dojo_types::schema::{Enum, Member, Struct, Ty};
use starknet::accounts::ConnectedAccount;
use starknet::core::types::{BlockId, BlockTag, FieldElement};

use crate::contract::world::test::deploy_world;
use crate::contract::world::WorldContractReader;

#[tokio::test(flavor = "multi_thread")]
async fn test_model() {
    let sequencer =
        TestSequencer::start(SequencerConfig::default(), get_default_test_starknet_config()).await;
    let account = sequencer.account();
    let provider = account.provider();
    let (world_address, _) = deploy_world(
        &sequencer,
        Utf8PathBuf::from_path_buf("../../../examples/ecs/target/dev".into()).unwrap(),
    )
    .await;

    let block_id = BlockId::Tag(BlockTag::Latest);
    let world = WorldContractReader::new(world_address, provider);
    let position = world.model("Position", block_id).await.unwrap();
    let schema = position.schema(block_id).await.unwrap();

    assert_eq!(
        schema,
        Ty::Struct(Struct {
            name: "Position".to_string(),
            children: vec![
                Member {
                    name: "player".to_string(),
                    ty: Ty::Primitive(CairoType::ContractAddress(None)),
                    key: true
                },
                Member {
                    name: "vec".to_string(),
                    ty: Ty::Struct(Struct {
                        name: "Vec2".to_string(),
                        children: vec![
                            Member {
                                name: "x".to_string(),
                                ty: Ty::Primitive(CairoType::U32(None)),
                                key: false
                            },
                            Member {
                                name: "y".to_string(),
                                ty: Ty::Primitive(CairoType::U32(None)),
                                key: false
                            }
                        ]
                    }),
                    key: false
                }
            ]
        })
    );

    assert_eq!(
        position.class_hash(),
        FieldElement::from_hex_be(
            "0x003598b0816df38211a0ebd6edcd922d19738778de2240caf6b04c6a0cab6df5"
        )
        .unwrap()
    );

    let moves = world.model("Moves", block_id).await.unwrap();
    let schema = moves.schema(block_id).await.unwrap();

    assert_eq!(
        schema,
        Ty::Struct(Struct {
            name: "Moves".to_string(),
            children: vec![
                Member {
                    name: "player".to_string(),
                    ty: Ty::Primitive(CairoType::ContractAddress(None)),
                    key: true
                },
                Member {
                    name: "remaining".to_string(),
                    ty: Ty::Primitive(CairoType::U8(None)),
                    key: false
                },
                Member {
                    name: "last_direction".to_string(),
                    ty: Ty::Enum(Enum {
                        name: "Direction".to_string(),
                        children: vec![
                            ("None".to_string(), Ty::Tuple(vec![])),
                            ("Left".to_string(), Ty::Tuple(vec![])),
                            ("Right".to_string(), Ty::Tuple(vec![])),
                            ("Up".to_string(), Ty::Tuple(vec![])),
                            ("Down".to_string(), Ty::Tuple(vec![]))
                        ]
                    }),
                    key: false
                }
            ]
        })
    );
}
