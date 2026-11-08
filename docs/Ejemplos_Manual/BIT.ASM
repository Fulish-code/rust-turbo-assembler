;**********************************************************
; BIT.ASM
;
; Este programa determina el valor del i-ésimo bit de una
; variable de un byte. Si el bit vale 0 deja un cero en la
; variable resul. Si el bit vale 1 deja un valor diferente
; de cero. El pseudocódigo de este programa es el
; siguiente:
;
;   AH = dato
;   AL = 1 << CL
;   AH = AH & AL
;   resul = AH
;**********************************************************

;****** CÓDIGO DE INICIO **********************************

        ideal
        dosseg
        model   small
        stack   256

;****** VARIABLES DEL PROGRAMA ****************************

        dataseg
codsal  db      0
dato    db      ?
bit     db      ?
resul   db      ?

;****** CÓDIGO DEL PROGRAMA *******************************

        codeseg
inicio:
        mov     ax, @data               ; Inicializa el

        mov     ds, ax                  ; segmento de datos

        mov     ah, [dato]              ; AH = dato
        mov     al, 1                   ; Al = 1
        mov     cl, [bit]               ; CL = bit
        shl     al, cl                  ; AL <= CL
        and     ah, al                  ; AH &= Al
        mov     [resul], ah             ; resul = AH

salir:
        mov     ah, 04Ch
        mov     al, [codsal]
        int     21h

;****** CÓDIGO DE TERMINACIÓN *****************************

        end     inicio
