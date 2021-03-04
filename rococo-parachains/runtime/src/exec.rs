// This trait will probably move to frame-support soon.
use frame_executive::ExecuteBlock;
use sp_api::{BlockT, HeaderT};

pub struct BlockExecutor<T, I>(sp_std::marker::PhantomData<(T, I)>);

impl<Block, T, I> ExecuteBlock<Block> for BlockExecutor<T, I>
where
	Block: BlockT,
	I: ExecuteBlock<Block>,
{
	fn execute_block(block: Block) {
		let (mut header, extrinsics) = block.deconstruct();

        // Seriously!? I can't fucking print here? And I can't gdb because it's wasm.
        // https://github.com/rust-lang/rust/issues/57966
        // info!("in runtime api impl. Initial digests are {:?}", header.digest());

		// let mut seal = None;
		header.digest_mut().logs.retain(|s| {
            //TODO, the real digest filtering logic will go here. But for starters, let's just try
            // to remove all the digests. There is only one anyway.
            false

			// match (s, seal.is_some()) {
			// 	(Some(_), true) => panic!("Found multiple AuRa seal digests"),
			// 	(None, _) => true,
			// 	(Some(s), false) => {
			// 		seal = Some(s);
			// 		false
			// 	}
			// }
		});

		I::execute_block(Block::new(header, extrinsics));

        //TODO eventually, I'll want to reconstruct the original and confirm the digests match.
        // I'll wait for https://github.com/paritytech/substrate/commits/bkchr-inherent-something-future
        // before I bother. Let's just get something working for now.
	}
}
