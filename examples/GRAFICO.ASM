; GRAFICO.ASM - Modo grafico 13h (320x200, 256 colores): franjas de colores
.MODEL SMALL
.STACK 100h

.CODE
inicio:
    mov ax, 0013h          ; entrar a modo 13h
    int 10h

    mov ax, 0A000h         ; memoria de video
    mov es, ax
    xor di, di
    xor dx, dx             ; fila actual
fila:
    mov cx, 320
    mov al, dl             ; color = fila
    rep stosb
    inc dx
    cmp dx, 200
    jb  fila

    mov ah, 00h            ; esperar una tecla
    int 16h

    mov ax, 0003h          ; volver a modo texto
    int 10h
    mov ax, 4C00h
    int 21h
END inicio
