;**********************************************************
; MAYOR3.ASM
;
; Este programa encuentra el mayor de tres datos signados
; almacenados en variables de una palabra. El pseudocódigo
; de este programa es:
;
;   AX = dato1
;   if(AX > dato2) goto sigcmp
;   AX = dato2
;
;sigcmp:
;   if(AX > dato3) goto fincmp
;   AX = dato3
;
;   mayor = AX

;**********************************************************

;****** CÓDIGO DE INICIO **********************************

        ideal
        dosseg
        model   small
        stack   256

;****** VARIABLES DEL PROGRAMA ****************************

        dataseg
codsal  db      0
dato1   dw      ?
dato2   dw      ?
dato3   dw      ?
mayor   dw      ?

;****** CÓDIGO DEL PROGRAMA *******************************

        codeseg
inicio:
        mov     ax, @data               ; Inicializa el
        mov     ds, ax                  ; segmento de datos

        mov     ax, [dato1]             ; AX = dato1
        cmp     ax, [dato2]             ; if(AX > dato2)
        jg      sigcmp                  ;    goto sigcmp
        mov     ax, [dato2]             ; AX = dato2

sigcmp: cmp     ax, [dato3]             ; if(AX > dato3)
        jg      fincmp                  ;    goto fincmp
        mov     ax, [dato3]             ; AX = dato3

fincmp: mov     [mayor], ax             ; mayor = AX

salir:
        mov     ah, 04Ch
        mov     al, [codsal]
        int     21h

;****** CÓDIGO DE TERMINACIÓN *****************************

        end     inicio
