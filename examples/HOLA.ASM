; HOLA.ASM - Hola mundo en DOS (16 bits)
; Ensamblar: TASM /zi HOLA.ASM   Enlazar: TLINK /v HOLA.OBJ
.MODEL SMALL
.STACK 100h

.DATA
    mensaje DB 'Hola, mundo desde Turbo Assembler!', 13, 10, '$'

.CODE
inicio:
    mov ax, @data          ; cargar el segmento de datos
    mov ds, ax

    mov ah, 09h            ; DOS: imprimir cadena terminada en '$'
    lea dx, mensaje
    int 21h

    mov ax, 4C00h          ; DOS: terminar el programa (codigo 0)
    int 21h
END inicio
