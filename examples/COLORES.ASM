;**********************************************************
; COLORES.ASM
;
; Este programa despliega un texto en 15 colores usando la
; función 13h de la interrupción 10h del BIOS (escribir una
; cadena). Al terminar espera una tecla.
;**********************************************************

;****** CÓDIGO DE INICIO **********************************

        ideal
        dosseg
        model   small
        stack   256

;****** VARIABLES DEL PROGRAMA ****************************

        dataseg
codsal  db      0
texto   db      'Turbo Assembler'
largo   =       $ - texto               ; Longitud del texto

;****** CÓDIGO DEL PROGRAMA *******************************

        codeseg
inicio:
        mov     ax, @data               ; Inicializa el
        mov     ds, ax                  ; segmento de datos y
        mov     es, ax                  ; el segmento extra

        mov     ax, 0003h               ; Modo texto 80x25,
        int     10h                     ; limpia la pantalla

        mov     bl, 1                   ; BL = color inicial
        mov     dh, 2                   ; DH = renglón
fila:   mov     ax, 1300h               ; Escribe la cadena
        mov     bh, 0                   ; en la página 0
        mov     cx, largo
        mov     dl, 30                  ; DL = columna
        mov     bp, offset texto        ; ES:BP = texto
        int     10h
        inc     dh                      ; Siguiente renglón
        inc     bl                      ; Siguiente color
        cmp     bl, 16
        jb      fila

        mov     ah, 00h                 ; Espera una tecla
        int     16h

salir:
        mov     ah, 04Ch
        mov     al, [codsal]
        int     21h

;****** CÓDIGO DE TERMINACIÓN *****************************

        end     inicio
