bits 32
global _start

; ============================================================
;  Multiboot2 Header（纯文本，无帧缓冲请求）
; ============================================================
section .multiboot_header
align 8

header_start:
    dd 0xE85250D6                     ; Multiboot2 魔数
    dd 0                              ; 架构（0 = 保护模式，i386）
    dd header_end - header_start      ; header 总长度
    dd -(0xE85250D6 + 0 + (header_end - header_start)) ; 校验和

    ; ----- 结束 tag -----
    align 8
    dw 0                              ; type = 0（结束）
    dw 0                              ; flags
    dd 8                              ; size = 8
header_end:

; ============================================================
;  32 位入口
; ============================================================
section .text
_start:
    ; eax = 魔数 (Multiboot2 应为 0x36D76289)
    ; ebx = 指向 Multiboot2 信息结构的指针
    mov edi, eax
    mov esi, ebx

    cmp eax, 0x36D76289               ; 验证 Multiboot2 引导
    jne .no_multiboot

    mov esp, stack_top

    call setup_paging
    call enable_long_mode
    lgdt [gdt64_ptr]
    jmp 0x08:long_mode_start

.no_multiboot:
    cli
    hlt

; ------------------------------------------------------------
;  恒等映射 0~2 MB（使用 2MB 大页）
; ------------------------------------------------------------
setup_paging:
    mov edi, pml4_table
    mov ecx, 0x1000 * 3
    xor eax, eax
    rep stosb

    mov dword [pml4_table], pdp_table + 0x3
    mov dword [pdp_table], pd_table + 0x3
    mov dword [pd_table], 0x0 | 0x83  ; 大页，存在，可写
    ret

; ------------------------------------------------------------
;  启用长模式（PAE + LME + PG）
; ------------------------------------------------------------
enable_long_mode:
    mov eax, cr4
    or eax, 0xA0                      ; PAE + PGE
    mov cr4, eax

    mov eax, pml4_table
    mov cr3, eax

    mov ecx, 0xC0000080
    rdmsr
    or eax, 0x100                     ; LME
    wrmsr

    mov eax, cr0
    or eax, 0x80000000                ; PG
    mov cr0, eax
    ret

; ============================================================
;  64 位代码
; ============================================================
bits 64
long_mode_start:
    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    mov ss, ax
    mov rsp, stack_top_64

    ; 传参给内核：
    ; rdi = 魔数（0x36D76289）
    ; rsi = Multiboot2 信息表指针
    mov rdi, rdi
    mov rsi, rsi

    extern kernel_main
    call kernel_main

    cli
    hlt

; ============================================================
;  GDT（64 位段）
; ============================================================
section .data
gdt64:
    dq 0
    dq (1<<43) | (1<<44) | (1<<47) | (1<<53)   ; 代码段 0x08
    dq (1<<44) | (1<<47) | (1<<41)              ; 数据段 0x10
gdt64_ptr:
    dw $ - gdt64 - 1
    dq gdt64

; ============================================================
;  页表（BSS，4K 对齐）
; ============================================================
section .bss
align 4096
pml4_table: resb 4096
pdp_table:  resb 4096
pd_table:   resb 4096

stack_bottom:
    resb 16384
stack_top:

stack_bottom_64:
    resb 16384
stack_top_64: