;**********************************************************
; HOLA.ASM
;
; Este programa despliega un saludo en la pantalla usando
; la función 09h de la interrupción 21h del DOS.
;**********************************************************

;****** CÓDIGO DE INICIO **********************************

        ideal
        dosseg
        model   small
        stack   256

;****** VARIABLES DEL PROGRAMA ****************************

        dataseg
codsal  db      0
mensaje db      'Hola, mundo desde Turbo Assembler!', 13, 10, '$'

;****** CÓDIGO DEL PROGRAMA *******************************

        codeseg
inicio:
        mov     ax, @data               ; Inicializa el
        mov     ds, ax                  ; segmento de datos

        mov     ah, 09h                 ; Despliega la cadena
        mov     dx, offset mensaje      ; terminada en '$'
        int     21h

salir:
        mov     ah, 04Ch
        mov     al, [codsal]
        int     21h

;****** CÓDIGO DE TERMINACIÓN *****************************

        end     inicio
