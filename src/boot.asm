bits 32
global _start

; ============================================================
;  Multiboot2 Header（请求线性帧缓冲）
; ============================================================
section .multiboot_header
align 8

header_start:
    dd 0xE85250D6                     ; Multiboot2 魔数
    dd 0                              ; 架构 (0 = i386 保护模式)
    dd header_end - header_start      ; header 总长度
    dd -(0xE85250D6 + 0 + (header_end - header_start)) ; 校验和

    ; ----- 帧缓冲请求 tag -----
align 8
framebuffer_tag_start:
    dw 5                              ; tag 类型: 5 = 帧缓冲请求
    dw 1                              ; flags: 1 = 强制要求图形模式（若不可用则启动失败）
    dd framebuffer_tag_end - framebuffer_tag_start ; tag 大小（此处固定 20）
    dd 1024                           ; 宽度
    dd 768                            ; 高度
    dd 32                             ; 深度 (BPP)
framebuffer_tag_end:

    ; ----- 结束 tag -----
align 8
    dw 0                              ; type = 0
    dw 0
    dd 8
header_end:

; ============================================================
;  32 位入口
; ============================================================
section .text
_start:
	out 0x3F8, 'H'
    out 0x3f8, 'i'
    cmp eax, 0x36D76289               ; 检查 Multiboot2 魔数
    jne .no_multiboot

    mov edi, eax                      ; 保存魔数（低 32 位）
    mov esi, ebx                      ; 保存 Multiboot2 信息指针（低 32 位）
    mov esp, stack_top                ; 临时栈（32 位）

    call setup_paging
    call enable_long_mode
    lgdt [gdt64_ptr]
    jmp 0x08:long_mode_start          ; 远跳转进入 64 位模式

.no_multiboot:
    cli
    hlt

; ------------------------------------------------------------
;  恒等映射 0 ~ 2 MB（2 MiB 大页）
; ------------------------------------------------------------
setup_paging:
    mov edi, pml4_table
    mov ecx, 0x1000 * 3
    xor eax, eax
    rep stosb

    mov dword [pml4_table], pdp_table + 0x3
    mov dword [pdp_table], pd_table + 0x3
    mov dword [pd_table], 0x0 | 0x83    ; 大页，基址 0，存在，可写
    ret

; ------------------------------------------------------------
;  启用长模式
; ------------------------------------------------------------
enable_long_mode:
    mov eax, cr4
    or eax, 0xA0                       ; PAE + PGE
    mov cr4, eax

    mov eax, pml4_table
    mov cr3, eax

    mov ecx, 0xC0000080
    rdmsr
    or eax, 0x100                      ; LME
    wrmsr

    mov eax, cr0
    or eax, 0x80000000                 ; PG
    mov cr0, eax
    ret

; ============================================================
;  64 位代码
; ============================================================
bits 64
long_mode_start:
    ; 加载 64 位数据段选择子
    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    mov ss, ax
    mov rsp, stack_top_64

    ; ── 关键修复：清零 rdi / rsi 的高 32 位 ──
    ; 32 位传送指令自动将目标 64 位寄存器的高位清零
    mov edi, edi          ; rdi = 0x00000000_36D76289
    mov esi, esi          ; rsi = 零扩展的 Multiboot2 信息指针

    ; 调用 Rust 内核
    extern kernel_main
    call kernel_main

    cli
    hlt

; ============================================================
;  GDT（64 位段描述符）
; ============================================================
section .data
gdt64:
    dq 0                                           ; 空选择子
    dq (1<<43) | (1<<44) | (1<<47) | (1<<53)      ; 代码段 0x08
    dq (1<<44) | (1<<47) | (1<<41)                 ; 数据段 0x10
gdt64_ptr:
    dw $ - gdt64 - 1
    dq gdt64

; ============================================================
;  页表空间（BSS，4096 对齐）
; ============================================================
section .bss
align 4096
pml4_table: resb 4096
pdp_table:  resb 4096
pd_table:   resb 4096

; 32 位模式临时栈
stack_bottom:
    resb 16384
stack_top:

; 64 位模式栈（对齐 16 字节）
align 16
stack_bottom_64:
    resb 65536
stack_top_64: