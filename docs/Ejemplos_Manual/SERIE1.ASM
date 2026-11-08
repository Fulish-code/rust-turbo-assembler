;**********************************************************
; SERIE1.ASM
;
; Este programa suma los números enteros de 1 hasta nfinal.
; El pseudocódigo de este programa es:
;
;   AX = 0
;   CX = nfinal
;
;   while(CX > 0)
;   {
;      AX += CX
;      CX--
;   }
;
;   suma = AX
;**********************************************************

;****** CÓDIGO DE INICIO **********************************

        ideal
        dosseg
        model   small
        stack   256

;****** VARIABLES DEL PROGRAMA ****************************

        dataseg
codsal  db      0
nfinal  dw      ?
suma    dw      ?

;****** CÓDIGO DEL PROGRAMA *******************************

        codeseg
inicio:
        mov     ax, @data               ; inicializa el
        mov     ds, ax                  ; segmento de datos

        xor     ax, ax                  ; AX = 0
        mov     cx, [nfinal]            ; CX = nfinal

;** CORREGIDO: el manual usa la etiqueta while; en TASM 4.1 WHILE es una
;** directiva (bloque de repetición), así que la etiqueta se llama whi.
whi:     jcxz    endwhi                  ; while(CX > 0)
                                        ; {
        add     ax, cx                  ;    AX += CX
        dec     cx                      ;    CX--
        jmp     whi                     ; }

endwhi:
        mov     [suma], ax              ; suma = AX

salir:
        mov     ah, 04Ch
        mov     al, [codsal]
        int     21h

;****** CÓDIGO DE TERMINACIÓN *****************************

        end     inicio
