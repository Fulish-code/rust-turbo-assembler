;**********************************************************
; GRAFICO.ASM
;
; Este programa entra al modo gráfico 13h (320x200, 256
; colores) y dibuja una franja de color por renglón. Al
; oprimir una tecla regresa al modo texto.
;**********************************************************

;****** CÓDIGO DE INICIO **********************************

        ideal
        dosseg
        model   small
        stack   256

;****** VARIABLES DEL PROGRAMA ****************************

        dataseg
codsal  db      0

;****** CÓDIGO DEL PROGRAMA *******************************

        codeseg
inicio:
        mov     ax, @data               ; Inicializa el
        mov     ds, ax                  ; segmento de datos

        mov     ax, 0013h               ; Entra al modo 13h
        int     10h

        mov     ax, 0A000h              ; ES = memoria de video
        mov     es, ax
        xor     di, di                  ; DI = 0
        xor     dx, dx                  ; DX = renglón
fila:   mov     cx, 320                 ; Llena un renglón
        mov     al, dl                  ; color = renglón
        cld
        rep     stosb
        inc     dx                      ; Siguiente renglón
        cmp     dx, 200
        jb      fila

        mov     ah, 00h                 ; Espera una tecla
        int     16h

        mov     ax, 0003h               ; Regresa al modo texto
        int     10h

salir:
        mov     ah, 04Ch
        mov     al, [codsal]
        int     21h

;****** CÓDIGO DE TERMINACIÓN *****************************

        end     inicio
