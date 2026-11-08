;**********************************************************
; SUMABYWS.ASM
;
; Este programa suma una variable de 1 byte con una
; variable de una palabra, ambas signadas. El pseudocódigo
; de este programa es el siguiente:
;
;   AL = dato1
;   AX = (int)AL
;   AX += dato2
;   resul = AX
;**********************************************************

;****** CÓDIGO DE INICIO **********************************

        ideal
        dosseg
        model   small
        stack   256

;****** VARIABLES DEL PROGRAMA ****************************

        dataseg
codsal  db      0
dato1   db      ?
dato2   dw      ?
resul   dw      ?

;****** CÓDIGO DEL PROGRAMA *******************************

        codeseg
inicio:
        mov     ax, @data               ; Inicializa el
        mov     ds, ax                  ; segmento de datos

        mov     al, [dato1]             ; AL = dato1
        cbw                             ; AX = AL
        add     ax, [dato2]             ; AX += dato2
        mov     [resul], ax             ; resul = AX
salir:
        mov     ah, 04Ch
        mov     al, [codsal]
        int     21h

;****** CÓDIGO DE TERMINACIÓN *****************************

        end     inicio
