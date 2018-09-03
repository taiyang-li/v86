use cpu::BitSize;
use global_pointers;
use jit::JitContext;
use modrm;
use regs;
use tlb::{TLB_GLOBAL, TLB_HAS_CODE, TLB_NO_USER, TLB_READONLY, TLB_VALID};
use wasmgen::module_init;
use wasmgen::module_init::{WasmBuilder, WasmLocal};
use wasmgen::wasm_util::WasmBuf;

extern "C" {
    #[no_mangle]
    static mut mem8: *mut u8;
}

pub fn gen_set_previous_eip_offset_from_eip(builder: &mut WasmBuilder, n: u32) {
    let cs = &mut builder.instruction_body;
    cs.const_i32(global_pointers::PREVIOUS_IP as i32); // store address of previous ip
    cs.load_aligned_i32(global_pointers::INSTRUCTION_POINTER); // load ip
    if n != 0 {
        cs.const_i32(n as i32);
        cs.add_i32(); // add constant to ip value
    }
    cs.store_aligned_i32(0); // store it as previous ip
}

pub fn gen_increment_instruction_pointer(builder: &mut WasmBuilder, n: u32) {
    let cs = &mut builder.instruction_body;
    cs.const_i32(global_pointers::INSTRUCTION_POINTER as i32); // store address of ip

    cs.load_aligned_i32(global_pointers::INSTRUCTION_POINTER); // load ip

    cs.const_i32(n as i32);

    cs.add_i32();
    cs.store_aligned_i32(0); // store it back in
}

pub fn gen_relative_jump(builder: &mut WasmBuilder, n: i32) {
    // add n to instruction_pointer (without setting the offset as above)
    let instruction_body = &mut builder.instruction_body;
    instruction_body.const_i32(global_pointers::INSTRUCTION_POINTER as i32);
    instruction_body.load_aligned_i32(global_pointers::INSTRUCTION_POINTER);
    instruction_body.const_i32(n);
    instruction_body.add_i32();
    instruction_body.store_aligned_i32(0);
}

pub fn gen_increment_variable(builder: &mut WasmBuilder, variable_address: u32, n: i32) {
    builder
        .instruction_body
        .increment_variable(variable_address, n);
}

pub fn gen_increment_timestamp_counter(builder: &mut WasmBuilder, n: i32) {
    gen_increment_variable(builder, global_pointers::TIMESTAMP_COUNTER, n);
}

pub fn gen_increment_mem32(builder: &mut WasmBuilder, addr: u32) {
    builder.instruction_body.increment_mem32(addr)
}

pub fn gen_get_reg8(builder: &mut WasmBuilder, r: u32) {
    builder
        .instruction_body
        .const_i32(global_pointers::get_reg8_offset(r) as i32);
    builder.instruction_body.load_u8_from_stack(0);
}

pub fn gen_get_reg16(builder: &mut WasmBuilder, r: u32) {
    builder
        .instruction_body
        .const_i32(global_pointers::get_reg16_offset(r) as i32);
    builder.instruction_body.load_aligned_u16_from_stack(0);
}

pub fn gen_get_reg32(builder: &mut WasmBuilder, r: u32) {
    builder
        .instruction_body
        .const_i32(global_pointers::get_reg32_offset(r) as i32);
    builder.instruction_body.load_aligned_i32_from_stack(0);
}

/// sign-extend a byte value on the stack and leave it on the stack
pub fn sign_extend_i8(builder: &mut WasmBuilder) {
    builder.instruction_body.const_i32(24);
    builder.instruction_body.shl_i32();
    builder.instruction_body.const_i32(24);
    builder.instruction_body.shr_s_i32();
}

/// sign-extend a two byte value on the stack and leave it on the stack
pub fn sign_extend_i16(builder: &mut WasmBuilder) {
    builder.instruction_body.const_i32(16);
    builder.instruction_body.shl_i32();
    builder.instruction_body.const_i32(16);
    builder.instruction_body.shr_s_i32();
}

pub fn gen_fn0_const(ctx: &mut JitContext, name: &str) {
    let builder = &mut ctx.builder;
    let fn_idx = builder.get_fn_idx(name, module_init::FN0_TYPE_INDEX);
    builder.instruction_body.call_fn(fn_idx);
}

pub fn gen_fn0_const_ret(builder: &mut WasmBuilder, name: &str) {
    let fn_idx = builder.get_fn_idx(name, module_init::FN0_RET_TYPE_INDEX);
    builder.instruction_body.call_fn(fn_idx);
}

pub fn gen_fn1_const(ctx: &mut JitContext, name: &str, arg0: u32) {
    let builder = &mut ctx.builder;
    let fn_idx = builder.get_fn_idx(name, module_init::FN1_TYPE_INDEX);
    builder.instruction_body.const_i32(arg0 as i32);
    builder.instruction_body.call_fn(fn_idx);
}

pub fn gen_call_fn1_ret(builder: &mut WasmBuilder, name: &str) {
    // generates: fn( _ ) where _ must be left on the stack before calling this, and fn returns a value
    let fn_idx = builder.get_fn_idx(name, module_init::FN1_RET_TYPE_INDEX);
    builder.instruction_body.call_fn(fn_idx);
}

pub fn gen_fn2_const(ctx: &mut JitContext, name: &str, arg0: u32, arg1: u32) {
    let builder = &mut ctx.builder;
    let fn_idx = builder.get_fn_idx(name, module_init::FN2_TYPE_INDEX);
    builder.instruction_body.const_i32(arg0 as i32);
    builder.instruction_body.const_i32(arg1 as i32);
    builder.instruction_body.call_fn(fn_idx);
}

pub fn gen_call_fn1(builder: &mut WasmBuilder, name: &str) {
    // generates: fn( _ ) where _ must be left on the stack before calling this
    let fn_idx = builder.get_fn_idx(name, module_init::FN1_TYPE_INDEX);
    builder.instruction_body.call_fn(fn_idx);
}

pub fn gen_call_fn2(builder: &mut WasmBuilder, name: &str) {
    // generates: fn( _, _ ) where _ must be left on the stack before calling this
    let fn_idx = builder.get_fn_idx(name, module_init::FN2_TYPE_INDEX);
    builder.instruction_body.call_fn(fn_idx);
}

pub fn gen_call_fn2_ret(builder: &mut WasmBuilder, name: &str) {
    // generates: fn( _, _ ) where _ must be left on the stack before calling this, and fn returns a value
    let fn_idx = builder.get_fn_idx(name, module_init::FN2_RET_TYPE_INDEX);
    builder.instruction_body.call_fn(fn_idx);
}

pub fn gen_call_fn3(ctx: &mut JitContext, name: &str) {
    let builder = &mut ctx.builder;
    let fn_idx = builder.get_fn_idx(name, module_init::FN3_TYPE_INDEX);
    builder.instruction_body.call_fn(fn_idx);
}

pub fn gen_fn3_const(ctx: &mut JitContext, name: &str, arg0: u32, arg1: u32, arg2: u32) {
    let builder = &mut ctx.builder;
    let fn_idx = builder.get_fn_idx(name, module_init::FN3_TYPE_INDEX);
    builder.instruction_body.const_i32(arg0 as i32);
    builder.instruction_body.const_i32(arg1 as i32);
    builder.instruction_body.const_i32(arg2 as i32);
    builder.instruction_body.call_fn(fn_idx);
}

pub fn gen_modrm_fn0(ctx: &mut JitContext, name: &str) {
    // generates: fn( _ )
    let builder = &mut ctx.builder;
    let fn_idx = builder.get_fn_idx(name, module_init::FN1_TYPE_INDEX);
    builder.instruction_body.call_fn(fn_idx);
}

pub fn gen_modrm_fn1(ctx: &mut JitContext, name: &str, arg0: u32) {
    // generates: fn( _, arg0 )
    let builder = &mut ctx.builder;
    let fn_idx = builder.get_fn_idx(name, module_init::FN2_TYPE_INDEX);
    builder.instruction_body.const_i32(arg0 as i32);
    builder.instruction_body.call_fn(fn_idx);
}

pub fn gen_modrm_fn2(ctx: &mut JitContext, name: &str, arg0: u32, arg1: u32) {
    // generates: fn( _, arg0, arg1 )
    let builder = &mut ctx.builder;
    let fn_idx = builder.get_fn_idx(name, module_init::FN3_TYPE_INDEX);
    builder.instruction_body.const_i32(arg0 as i32);
    builder.instruction_body.const_i32(arg1 as i32);
    builder.instruction_body.call_fn(fn_idx);
}

pub fn gen_modrm_resolve(ctx: &mut JitContext, modrm_byte: u8) { modrm::gen(ctx, modrm_byte) }

pub fn gen_set_reg8_r(ctx: &mut JitContext, dest: u32, src: u32) {
    // generates: reg8[r_dest] = reg8[r_src]
    let builder = &mut ctx.builder;
    builder
        .instruction_body
        .const_i32(global_pointers::get_reg8_offset(dest) as i32);
    gen_get_reg8(builder, src);
    builder.instruction_body.store_u8(0);
}
pub fn gen_set_reg16_r(ctx: &mut JitContext, dest: u32, src: u32) {
    // generates: reg16[r_dest] = reg16[r_src]
    let builder = &mut ctx.builder;
    builder
        .instruction_body
        .const_i32(global_pointers::get_reg16_offset(dest) as i32);
    gen_get_reg16(builder, src);
    builder.instruction_body.store_aligned_u16(0);
}
pub fn gen_set_reg32_r(ctx: &mut JitContext, dest: u32, src: u32) {
    // generates: reg32s[r_dest] = reg32s[r_src]
    let builder = &mut ctx.builder;
    builder
        .instruction_body
        .const_i32(global_pointers::get_reg32_offset(dest) as i32);
    gen_get_reg32(builder, src);
    builder.instruction_body.store_aligned_i32(0);
}

pub fn gen_safe_read8(ctx: &mut JitContext) { gen_safe_read(ctx, BitSize::BYTE) }
pub fn gen_safe_read16(ctx: &mut JitContext) { gen_safe_read(ctx, BitSize::WORD) }
pub fn gen_safe_read32(ctx: &mut JitContext) { gen_safe_read(ctx, BitSize::DWORD) }

pub fn gen_safe_write8(ctx: &mut JitContext, address_local: &WasmLocal, value_local: &WasmLocal) {
    gen_safe_write(ctx, BitSize::BYTE, address_local, value_local)
}
pub fn gen_safe_write16(ctx: &mut JitContext, address_local: &WasmLocal, value_local: &WasmLocal) {
    gen_safe_write(ctx, BitSize::WORD, address_local, value_local)
}
pub fn gen_safe_write32(ctx: &mut JitContext, address_local: &WasmLocal, value_local: &WasmLocal) {
    gen_safe_write(ctx, BitSize::DWORD, address_local, value_local)
}

fn gen_safe_read(ctx: &mut JitContext, bits: BitSize) {
    // Assumes virtual address has been pushed to the stack, and generates safe_readXX's fast-path
    // inline, bailing to safe_readXX_slow if necessary
    let builder = &mut ctx.builder;

    let address_local = builder.tee_new_local();

    // Pseudo: base_on_stack = (uint32_t)address >> 12;
    builder.instruction_body.const_i32(12);
    builder.instruction_body.shr_u_i32();

    // scale index
    builder.instruction_body.const_i32(2);
    builder.instruction_body.shl_i32();

    // Pseudo: entry = tlb_data[base_on_stack];
    builder
        .instruction_body
        .load_aligned_i32_from_stack(global_pointers::TLB_DATA);
    let entry_local = builder.tee_new_local();

    // Pseudo: bool can_use_fast_path =
    //    (entry & 0xFFF & ~TLB_READONLY & ~TLB_GLOBAL & ~TLB_HAS_CODE & ~(cpl == 3 ? 0 : TLB_NO_USER) == TLB_VALID &&
    //    (bitsize == 8 ? true : (address & 0xFFF) <= (0x1000 - (bitsize / 8)));
    builder.instruction_body.const_i32(
        (0xFFF
            & !TLB_READONLY
            & !TLB_GLOBAL
            & !TLB_HAS_CODE
            & !(if ctx.cpu.cpl3() { 0 } else { TLB_NO_USER })) as i32,
    );
    builder.instruction_body.and_i32();

    builder.instruction_body.const_i32(TLB_VALID as i32);
    builder.instruction_body.eq_i32();

    if bits != BitSize::BYTE {
        builder.instruction_body.get_local(&address_local);
        builder.instruction_body.const_i32(0xFFF);
        builder.instruction_body.and_i32();
        builder
            .instruction_body
            .const_i32(0x1000 - if bits == BitSize::WORD { 2 } else { 4 });
        builder.instruction_body.le_i32();

        builder.instruction_body.and_i32();
    }

    // Pseudo:
    // if(can_use_fast_path) leave_on_stack(mem8[entry & ~0xFFF ^ address]);
    builder.instruction_body.if_i32();
    builder.instruction_body.get_local(&entry_local);
    builder.instruction_body.const_i32(!0xFFF);
    builder.instruction_body.and_i32();
    builder.instruction_body.get_local(&address_local);
    builder.instruction_body.xor_i32();

    match bits {
        BitSize::BYTE => {
            builder
                .instruction_body
                .load_u8_from_stack(unsafe { mem8 } as u32);
        },
        BitSize::WORD => {
            builder
                .instruction_body
                .load_unaligned_u16_from_stack(unsafe { mem8 } as u32);
        },
        BitSize::DWORD => {
            builder
                .instruction_body
                .load_unaligned_i32_from_stack(unsafe { mem8 } as u32);
        },
    }

    // Pseudo:
    // else {
    //     *previous_ip = *instruction_pointer & ~0xFFF | start_of_instruction;
    //     leave_on_stack(safe_read*_slow(address));
    //     if(page_fault) return;
    // }
    builder.instruction_body.else_();

    builder
        .instruction_body
        .const_i32(global_pointers::PREVIOUS_IP as i32);

    builder
        .instruction_body
        .load_aligned_i32(global_pointers::INSTRUCTION_POINTER);
    builder.instruction_body.const_i32(!0xFFF);
    builder.instruction_body.and_i32();
    builder
        .instruction_body
        .const_i32(ctx.start_of_current_instruction as i32 & 0xFFF);
    builder.instruction_body.or_i32();

    builder.instruction_body.store_aligned_i32(0);

    builder.instruction_body.get_local(&address_local);
    match bits {
        BitSize::BYTE => {
            gen_call_fn1_ret(builder, "safe_read8_slow_jit");
        },
        BitSize::WORD => {
            gen_call_fn1_ret(builder, "safe_read16_slow_jit");
        },
        BitSize::DWORD => {
            gen_call_fn1_ret(builder, "safe_read32s_slow_jit");
        },
    }

    builder
        .instruction_body
        .load_u8(global_pointers::PAGE_FAULT);

    builder.instruction_body.if_void();
    builder.instruction_body.return_();
    builder.instruction_body.block_end();

    builder.instruction_body.block_end();

    builder.free_local(address_local);
    builder.free_local(entry_local);
}

fn gen_safe_write(
    ctx: &mut JitContext,
    bits: BitSize,
    address_local: &WasmLocal,
    value_local: &WasmLocal,
) {
    // Generates safe_writeXX' fast-path inline, bailing to safe_writeXX_slow if necessary.

    let builder = &mut ctx.builder;

    builder.instruction_body.get_local(&address_local);

    // Pseudo: base_on_stack = (uint32_t)address >> 12;
    builder.instruction_body.const_i32(12);
    builder.instruction_body.shr_u_i32();

    // scale index
    builder.instruction_body.const_i32(2);
    builder.instruction_body.shl_i32();

    // Pseudo: entry = tlb_data[base_on_stack];
    builder
        .instruction_body
        .load_aligned_i32_from_stack(global_pointers::TLB_DATA);
    let entry_local = builder.tee_new_local();

    // Pseudo: bool can_use_fast_path = (entry & 0xFFF & ~TLB_GLOBAL & ~(cpl == 3 ? 0 : TLB_NO_USER) == TLB_VALID &&
    //                                   (address & 0xFFF) <= (0x1000 - bitsize / 8));
    builder
        .instruction_body
        .const_i32((0xFFF & !TLB_GLOBAL & !(if ctx.cpu.cpl3() { 0 } else { TLB_NO_USER })) as i32);
    builder.instruction_body.and_i32();

    builder.instruction_body.const_i32(TLB_VALID as i32);
    builder.instruction_body.eq_i32();

    if bits != BitSize::BYTE {
        builder.instruction_body.get_local(&address_local);
        builder.instruction_body.const_i32(0xFFF);
        builder.instruction_body.and_i32();
        builder
            .instruction_body
            .const_i32(0x1000 - if bits == BitSize::WORD { 2 } else { 4 });
        builder.instruction_body.le_i32();

        builder.instruction_body.and_i32();
    }

    // Pseudo:
    // if(can_use_fast_path)
    // {
    //     phys_addr = entry & ~0xFFF ^ address;
    builder.instruction_body.if_void();

    builder.instruction_body.get_local(&entry_local);
    builder.instruction_body.const_i32(!0xFFF);
    builder.instruction_body.and_i32();
    builder.instruction_body.get_local(&address_local);
    builder.instruction_body.xor_i32();

    builder.free_local(entry_local);

    // Pseudo:
    //     /* continued within can_use_fast_path branch */
    //     mem8[phys_addr] = value;

    builder.instruction_body.get_local(&value_local);
    match bits {
        BitSize::BYTE => {
            builder.instruction_body.store_u8(unsafe { mem8 } as u32);
        },
        BitSize::WORD => {
            builder
                .instruction_body
                .store_unaligned_u16(unsafe { mem8 } as u32);
        },
        BitSize::DWORD => {
            builder
                .instruction_body
                .store_unaligned_i32(unsafe { mem8 } as u32);
        },
    }

    // Pseudo:
    // else {
    //     *previous_ip = *instruction_pointer & ~0xFFF | start_of_instruction;
    //     safe_write*_slow(address, value);
    //     if(page_fault) return;
    // }
    builder.instruction_body.else_();

    builder
        .instruction_body
        .const_i32(global_pointers::PREVIOUS_IP as i32);

    builder
        .instruction_body
        .load_aligned_i32(global_pointers::INSTRUCTION_POINTER);
    builder.instruction_body.const_i32(!0xFFF);
    builder.instruction_body.and_i32();
    builder
        .instruction_body
        .const_i32(ctx.start_of_current_instruction as i32 & 0xFFF);
    builder.instruction_body.or_i32();

    builder.instruction_body.store_aligned_i32(0);

    builder.instruction_body.get_local(&address_local);
    builder.instruction_body.get_local(&value_local);
    match bits {
        BitSize::BYTE => {
            gen_call_fn2(builder, "safe_write8_slow_jit");
        },
        BitSize::WORD => {
            gen_call_fn2(builder, "safe_write16_slow_jit");
        },
        BitSize::DWORD => {
            gen_call_fn2(builder, "safe_write32_slow_jit");
        },
    }

    builder
        .instruction_body
        .load_u8(global_pointers::PAGE_FAULT);

    builder.instruction_body.if_void();
    builder.instruction_body.return_();
    builder.instruction_body.block_end();

    builder.instruction_body.block_end();
}

pub fn gen_clear_prefixes(ctx: &mut JitContext) {
    let instruction_body = &mut ctx.builder.instruction_body;
    instruction_body.const_i32(global_pointers::PREFIXES as i32); // load address of prefixes
    instruction_body.const_i32(0);
    instruction_body.store_aligned_i32(0);
}

pub fn gen_add_prefix_bits(ctx: &mut JitContext, mask: u32) {
    dbg_assert!(mask < 0x100);

    let instruction_body = &mut ctx.builder.instruction_body;
    instruction_body.const_i32(global_pointers::PREFIXES as i32); // load address of prefixes

    instruction_body.load_aligned_i32(global_pointers::PREFIXES); // load old value
    instruction_body.const_i32(mask as i32);
    instruction_body.or_i32();

    instruction_body.store_aligned_i32(0);
}

pub fn gen_jmp_rel16(builder: &mut WasmBuilder, rel16: u16) {
    let cs_offset_addr = global_pointers::get_seg_offset(regs::CS);
    builder.instruction_body.load_aligned_i32(cs_offset_addr);
    let local = builder.set_new_local();

    // generate:
    // *instruction_pointer = cs_offset + ((*instruction_pointer - cs_offset + rel16) & 0xFFFF);
    {
        let instruction_body = &mut builder.instruction_body;

        instruction_body.const_i32(global_pointers::INSTRUCTION_POINTER as i32);

        instruction_body.load_aligned_i32(global_pointers::INSTRUCTION_POINTER);
        instruction_body.get_local(&local);
        instruction_body.sub_i32();

        instruction_body.const_i32(rel16 as i32);
        instruction_body.add_i32();

        instruction_body.const_i32(0xFFFF);
        instruction_body.and_i32();

        instruction_body.get_local(&local);
        instruction_body.add_i32();

        instruction_body.store_aligned_i32(0);
    }
    builder.free_local(local);
}

pub fn gen_pop16_ss16(ctx: &mut JitContext) {
    // sp = segment_offsets[SS] + reg16[SP] (or just reg16[SP] if has_flat_segmentation)
    gen_get_reg16(ctx.builder, regs::SP);
    let sp_local = ctx.builder.tee_new_local();

    if !ctx.cpu.has_flat_segmentation() {
        ctx.builder
            .instruction_body
            .load_aligned_i32(global_pointers::get_seg_offset(regs::SS));
        ctx.builder.instruction_body.add_i32();
    }

    // result = safe_read16(sp)
    gen_safe_read16(ctx);

    // reg16[SP] += 2;
    ctx.builder
        .instruction_body
        .const_i32(global_pointers::get_reg16_offset(regs::SP) as i32);
    ctx.builder.instruction_body.get_local(&sp_local);
    ctx.builder.instruction_body.const_i32(2);
    ctx.builder.instruction_body.add_i32();
    ctx.builder.instruction_body.store_aligned_u16(0);

    ctx.builder.free_local(sp_local);

    // return value is already on stack
}

pub fn gen_pop16_ss32(ctx: &mut JitContext) {
    // esp = segment_offsets[SS] + reg32s[ESP] (or just reg32s[ESP] if has_flat_segmentation)
    gen_get_reg32(ctx.builder, regs::ESP);
    let esp_local = ctx.builder.tee_new_local();

    if !ctx.cpu.has_flat_segmentation() {
        ctx.builder
            .instruction_body
            .load_aligned_i32(global_pointers::get_seg_offset(regs::SS));
        ctx.builder.instruction_body.add_i32();
    }

    // result = safe_read16(esp)
    gen_safe_read16(ctx);

    // reg32s[ESP] += 2;
    ctx.builder
        .instruction_body
        .const_i32(global_pointers::get_reg32_offset(regs::ESP) as i32);
    ctx.builder.instruction_body.get_local(&esp_local);
    ctx.builder.instruction_body.const_i32(2);
    ctx.builder.instruction_body.add_i32();
    ctx.builder.instruction_body.store_aligned_i32(0);
    ctx.builder.free_local(esp_local);

    // return value is already on stack
}

pub fn gen_pop16(ctx: &mut JitContext) {
    if ctx.cpu.ssize_32() {
        gen_pop16_ss32(ctx);
    }
    else {
        gen_pop16_ss16(ctx);
    }
}

pub fn gen_pop32s_ss16(ctx: &mut JitContext) {
    // sp = reg16[SP]
    gen_get_reg16(ctx.builder, regs::SP);
    let local_sp = ctx.builder.tee_new_local();

    // result = safe_read32s(segment_offsets[SS] + sp) (or just sp if has_flat_segmentation)
    if !ctx.cpu.has_flat_segmentation() {
        ctx.builder
            .instruction_body
            .load_aligned_i32(global_pointers::get_seg_offset(regs::SS));
        ctx.builder.instruction_body.add_i32();
    }

    gen_safe_read32(ctx);

    // reg16[SP] = sp + 4;
    ctx.builder
        .instruction_body
        .const_i32(global_pointers::get_reg16_offset(regs::SP) as i32);
    ctx.builder.instruction_body.get_local(&local_sp);
    ctx.builder.instruction_body.const_i32(4);
    ctx.builder.instruction_body.add_i32();
    ctx.builder.instruction_body.store_aligned_u16(0);

    ctx.builder.free_local(local_sp);

    // return value is already on stack
}

pub fn gen_pop32s_ss32(ctx: &mut JitContext) {
    // esp = reg32s[ESP]
    gen_get_reg32(ctx.builder, regs::ESP);
    let local_esp = ctx.builder.tee_new_local();

    // result = safe_read32s(segment_offsets[SS] + esp) (or just esp if has_flat_segmentation)
    if !ctx.cpu.has_flat_segmentation() {
        ctx.builder
            .instruction_body
            .load_aligned_i32(global_pointers::get_seg_offset(regs::SS));
        ctx.builder.instruction_body.add_i32();
    }
    gen_safe_read32(ctx);

    // reg32s[ESP] = esp + 4;
    ctx.builder
        .instruction_body
        .const_i32(global_pointers::get_reg32_offset(regs::ESP) as i32);
    ctx.builder.instruction_body.get_local(&local_esp);
    ctx.builder.instruction_body.const_i32(4);
    ctx.builder.instruction_body.add_i32();
    ctx.builder.instruction_body.store_aligned_i32(0);

    ctx.builder.free_local(local_esp);

    // return value is already on stack
}

pub fn gen_pop32s(ctx: &mut JitContext) {
    if ctx.cpu.ssize_32() {
        gen_pop32s_ss32(ctx);
    }
    else {
        gen_pop32s_ss16(ctx);
    }
}

pub fn gen_task_switch_test(ctx: &mut JitContext) {
    // generate if(cr[0] & (CR0_EM | CR0_TS)) { task_switch_test_void(); return; }

    let cr0_offset = global_pointers::get_creg_offset(0);

    ctx.builder.instruction_body.load_aligned_i32(cr0_offset);
    ctx.builder
        .instruction_body
        .const_i32((regs::CR0_EM | regs::CR0_TS) as i32);
    ctx.builder.instruction_body.and_i32();

    ctx.builder.instruction_body.if_void();

    gen_fn0_const(ctx, "task_switch_test_void");
    ctx.builder.instruction_body.return_();

    ctx.builder.instruction_body.block_end();
}

pub fn gen_task_switch_test_mmx(ctx: &mut JitContext) {
    // generate if(cr[0] & (CR0_EM | CR0_TS)) { task_switch_test_mmx_void(); return; }
    let cr0_offset = global_pointers::get_creg_offset(0);

    ctx.builder.instruction_body.load_aligned_i32(cr0_offset);
    ctx.builder
        .instruction_body
        .const_i32((regs::CR0_EM | regs::CR0_TS) as i32);
    ctx.builder.instruction_body.and_i32();

    ctx.builder.instruction_body.if_void();

    gen_fn0_const(ctx, "task_switch_test_mmx_void");
    ctx.builder.instruction_body.return_();

    ctx.builder.instruction_body.block_end();
}

pub fn gen_push16(ctx: &mut JitContext, value_local: &WasmLocal) {
    let sp_reg = if ctx.cpu.ssize_32() {
        gen_get_reg32(ctx.builder, regs::ESP);
        global_pointers::get_reg32_offset(regs::ESP)
    }
    else {
        gen_get_reg16(ctx.builder, regs::SP);
        global_pointers::get_reg16_offset(regs::SP)
    };

    ctx.builder.instruction_body.const_i32(2);
    ctx.builder.instruction_body.sub_i32();
    let reg_updated_local = ctx.builder.tee_new_local();
    if !ctx.cpu.ssize_32() {
        ctx.builder.instruction_body.const_i32(0xFFFF);
        ctx.builder.instruction_body.and_i32();
    }

    if !ctx.cpu.has_flat_segmentation() {
        ctx.builder
            .instruction_body
            .load_aligned_i32(global_pointers::get_seg_offset(regs::SS));
        ctx.builder.instruction_body.add_i32();
    }

    let sp_local = ctx.builder.set_new_local();
    gen_safe_write16(ctx, &sp_local, &value_local);
    ctx.builder.free_local(sp_local);
    ctx.builder.instruction_body.const_i32(sp_reg as i32);
    ctx.builder.instruction_body.get_local(&reg_updated_local);
    if ctx.cpu.ssize_32() {
        ctx.builder.instruction_body.store_aligned_i32(0)
    }
    else {
        ctx.builder.instruction_body.store_aligned_u16(0)
    };
    ctx.builder.free_local(reg_updated_local);
}

pub fn gen_push32(ctx: &mut JitContext, value_local: &WasmLocal) {
    let sp_reg = if ctx.cpu.ssize_32() {
        gen_get_reg32(ctx.builder, regs::ESP);
        global_pointers::get_reg32_offset(regs::ESP)
    }
    else {
        gen_get_reg16(ctx.builder, regs::SP);
        global_pointers::get_reg16_offset(regs::SP)
    };

    ctx.builder.instruction_body.const_i32(4);
    ctx.builder.instruction_body.sub_i32();
    if !ctx.cpu.ssize_32() {
        ctx.builder.instruction_body.const_i32(0xFFFF);
        ctx.builder.instruction_body.and_i32();
    }
    let new_sp_local = ctx.builder.tee_new_local();

    if !ctx.cpu.has_flat_segmentation() {
        ctx.builder
            .instruction_body
            .load_aligned_i32(global_pointers::get_seg_offset(regs::SS));
        ctx.builder.instruction_body.add_i32();
    }

    let sp_local = ctx.builder.set_new_local();

    gen_safe_write32(ctx, &sp_local, &value_local);
    ctx.builder.free_local(sp_local);

    ctx.builder.instruction_body.const_i32(sp_reg as i32);
    ctx.builder.instruction_body.get_local(&new_sp_local);
    if ctx.cpu.ssize_32() {
        ctx.builder.instruction_body.store_aligned_i32(0)
    }
    else {
        ctx.builder.instruction_body.store_aligned_u16(0)
    };
    ctx.builder.free_local(new_sp_local);
}

pub fn gen_get_real_eip(ctx: &mut JitContext) {
    let instruction_body = &mut ctx.builder.instruction_body;
    instruction_body.load_aligned_i32(global_pointers::INSTRUCTION_POINTER);
    instruction_body.load_aligned_i32(global_pointers::get_seg_offset(regs::CS));
    instruction_body.sub_i32();
}

pub fn gen_safe_read_write(
    ctx: &mut JitContext,
    bits: BitSize,
    address_local: &WasmLocal,
    f: &Fn(&mut JitContext),
    fallback_fn: &Fn(&mut JitContext),
) {
    ctx.builder.instruction_body.get_local(address_local);

    // Pseudo: base_on_stack = (uint32_t)address >> 12;
    ctx.builder.instruction_body.const_i32(12);
    ctx.builder.instruction_body.shr_u_i32();

    // scale index
    ctx.builder.instruction_body.const_i32(2);
    ctx.builder.instruction_body.shl_i32();

    // Pseudo: entry = tlb_data[base_on_stack];
    ctx.builder
        .instruction_body
        .load_aligned_i32_from_stack(global_pointers::TLB_DATA);
    let entry_local = ctx.builder.tee_new_local();

    // Pseudo: bool can_use_fast_path = (entry & 0xFFF & ~TLB_READONLY & ~TLB_GLOBAL & ~(cpl == 3 ? 0 : TLB_NO_USER) == TLB_VALID &&
    //                                   (address & 0xFFF) <= (0x1000 - (bitsize / 8));
    ctx.builder
        .instruction_body
        .const_i32((0xFFF & !TLB_GLOBAL & !(if ctx.cpu.cpl3() { 0 } else { TLB_NO_USER })) as i32);
    ctx.builder.instruction_body.and_i32();

    ctx.builder.instruction_body.const_i32(TLB_VALID as i32);
    ctx.builder.instruction_body.eq_i32();

    ctx.builder.instruction_body.get_local(&address_local);
    ctx.builder.instruction_body.const_i32(0xFFF);
    ctx.builder.instruction_body.and_i32();
    ctx.builder
        .instruction_body
        .const_i32(0x1000 - if bits == BitSize::WORD { 2 } else { 4 });
    ctx.builder.instruction_body.le_i32();

    ctx.builder.instruction_body.and_i32();

    // Pseudo:
    // if(can_use_fast_path) leave_on_stack(mem8[entry & ~0xFFF ^ address]);
    ctx.builder.instruction_body.if_void();
    ctx.builder.instruction_body.get_local(&entry_local);
    ctx.builder.instruction_body.const_i32(!0xFFF);
    ctx.builder.instruction_body.and_i32();
    ctx.builder.instruction_body.get_local(&address_local);
    ctx.builder.instruction_body.xor_i32();

    let phys_addr_local = ctx.builder.tee_new_local();
    // now store the value on stack to phys_addr
    ctx.builder.instruction_body.get_local(&phys_addr_local);

    match bits {
        BitSize::BYTE => {},
        BitSize::WORD => {
            ctx.builder
                .instruction_body
                .load_unaligned_u16_from_stack(unsafe { mem8 } as u32);
        },
        BitSize::DWORD => {
            ctx.builder
                .instruction_body
                .load_unaligned_i32_from_stack(unsafe { mem8 } as u32);
        },
    }

    ctx.builder.free_local(entry_local);

    f(ctx);

    match bits {
        BitSize::BYTE => {},
        BitSize::WORD => {
            ctx.builder
                .instruction_body
                .store_unaligned_u16(unsafe { mem8 } as u32);
        },
        BitSize::DWORD => {
            ctx.builder
                .instruction_body
                .store_unaligned_i32(unsafe { mem8 } as u32);
        },
    };
    ctx.builder.free_local(phys_addr_local);

    // Pseudo:
    // else {
    //     *previous_ip = *instruction_pointer & ~0xFFF | start_of_instruction;
    //     fallback_fn();
    //     if(page_fault) return;
    // }
    ctx.builder.instruction_body.else_();

    ctx.builder
        .instruction_body
        .const_i32(global_pointers::PREVIOUS_IP as i32);

    ctx.builder
        .instruction_body
        .load_aligned_i32(global_pointers::INSTRUCTION_POINTER);
    ctx.builder.instruction_body.const_i32(!0xFFF);
    ctx.builder.instruction_body.and_i32();
    ctx.builder
        .instruction_body
        .const_i32(ctx.start_of_current_instruction as i32 & 0xFFF);
    ctx.builder.instruction_body.or_i32();

    ctx.builder.instruction_body.store_aligned_i32(0);

    ctx.builder.instruction_body.get_local(&address_local);

    fallback_fn(ctx);

    ctx.builder
        .instruction_body
        .load_u8(global_pointers::PAGE_FAULT);

    ctx.builder.instruction_body.if_void();
    ctx.builder.instruction_body.return_();
    ctx.builder.instruction_body.block_end();

    ctx.builder.instruction_body.block_end();
}
