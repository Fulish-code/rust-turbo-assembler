; COLORES.ASM - Texto de colores con la BIOS (INT 10h)
.MODEL SMALL
.STACK 100h

.DATA
    texto DB 'Turbo Assembler'
    largo EQU $ - texto

.CODE
inicio:
    mov ax, @data
    mov ds, ax
    mov es, ax

    mov ax, 0003h          ; modo texto 80x25, limpia la pantalla
    int 10h

    mov bl, 1              ; atributo (color) inicial
    mov dh, 2              ; fila
filas:
    mov ax, 1300h          ; BIOS: escribir cadena
    mov bh, 0
    mov cx, largo
    mov dl, 30             ; columna
    lea bp, texto
    int 10h
    inc dh
    inc bl
    cmp bl, 16
    jb  filas

    mov ah, 00h            ; esperar una tecla
    int 16h
    mov ax, 4C00h
    int 21h
END inicio
