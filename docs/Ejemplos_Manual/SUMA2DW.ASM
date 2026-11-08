;**********************************************************
; SUMA2DW.ASM
;
; Este programa suma dos variables de tipo palabra doble.
; El resultado queda en una variable tipo palabra doble.
; El pseudocódigo de este programa es el siguiente:
;
;   DX:AX = dato1
;   DX:AX += dato2
;   resul = DX:AX
;**********************************************************

;****** CÓDIGO DE INICIO **********************************

        ideal
        dosseg
        model   small
        stack   256

;****** VARIABLES DEL PROGRAMA ****************************

        dataseg
codsal  db      0
dato1   dd      ?
dato2   dd      ?
resul   dd      ?

;****** CÓDIGO DEL PROGRAMA *******************************

        codeseg
inicio:
        mov     ax, @data               ; Inicializa el
        mov     ds, ax                  ; segmento de datos

        mov     ax, [word dato1]        ; DX:AX = dato1
        mov     dx, [word dato1+2]
        add     ax, [word dato2]        ; DX:AX += dato2
        adc     dx, [word dato2+2]
;** CORREGIDO: el manual dice [word resull]; la variable se llama resul.
        mov     [word resul], ax        ; resul = DX:AX
        mov     [word resul+2], dx
salir:
        mov     ah, 04Ch
        mov     al, [codsal]
        int     21h

;****** CÓDIGO DE TERMINACIÓN *****************************

        end     inicio
