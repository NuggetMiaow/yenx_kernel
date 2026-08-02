bits 32
global _start

; ─────────── Multiboot Header ───────────
section .multiboot_header
multiboot_header_start:
    dd 0x1BADB002          ; magic
    dd 0x0                 ; flags
    dd -(0x1BADB002 + 0x0) ; checksum
multiboot_header_end:

; ─────────── 32 位入口 ───────────
section .text
_start:
    mov edi, eax            ; magic
    mov esi, ebx            ; multiboot info pointer

    cmp eax, 0x2BADB002
    jne multiboot_error     ; 明确的标签，不用 .error

    mov esp, stack_top

    call setup_paging
    call enable_long_mode
    lgdt [gdt64_ptr]
    jmp 0x08:long_mode_start   ; 远跳转进入 64 位模式

multiboot_error:
    cli
    hlt

setup_paging:
    ; 清空 3 页 (PML4, PDP, PD)
    mov edi, pml4_table
    mov ecx, 0x1000 * 3
    xor eax, eax
    rep stosb

    ; PML4[0] → PDP
    mov dword [pml4_table], pdp_table + 0x3
    ; PDP[0] → PD (2MB 大页)
    mov dword [pdp_table], pd_table + 0x3
    ; PD[0] → 恒等映射 0~2MB，大页，可写，存在
    mov dword [pd_table], 0x0 | 0x83
    ret

enable_long_mode:
    mov eax, cr4
    or eax, 0xA0          ; PAE + PGE
    mov cr4, eax

    mov eax, pml4_table
    mov cr3, eax

    mov ecx, 0xC0000080   ; EFER MSR
    rdmsr
    or eax, 0x100         ; LME (Long Mode Enable)
    wrmsr

    mov eax, cr0
    or eax, 0x80000000    ; PG (Paging)
    mov cr0, eax
    ret

; ─────────── 64 位代码 ───────────
bits 64
long_mode_start:
    ; 加载数据段选择子 (0x10) 到所有数据段寄存器
    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    mov ss, ax
    mov rsp, stack_top_64

    ; 将 multiboot 参数传给 kernel_main
    mov rdi, rdi            ; 原 edi 已零扩到 rdi
    mov rsi, rsi            ; 原 esi 已零扩到 rsi

    extern kernel_main
    call kernel_main

    cli
    hlt

; ─────────── GDT ───────────
section .data
gdt64:
    dq 0                                      ; 空选择子 (0x00)
    dq (1<<43) | (1<<44) | (1<<47) | (1<<53) ; 代码段选择子 (0x08)
    dq (1<<44) | (1<<47) | (1<<41)           ; 数据段选择子 (0x10)
gdt64_ptr:
    dw $ - gdt64 - 1
    dq gdt64

; ─────────── 页表空间 ───────────
section .bss
align 4096
pml4_table: resb 4096
pdp_table:  resb 4096
pd_table:   resb 4096

; ─────────── 栈 ───────────
stack_bottom:
    resb 16384
stack_top:

stack_bottom_64:
    resb 16384
stack_top_64: